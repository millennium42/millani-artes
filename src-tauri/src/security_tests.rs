use std::collections::BTreeMap;
use tauri::ipc::Origin;
use tauri::utils::acl::{
    capability::{Capability, PermissionEntry},
    get_capabilities,
};

#[test]
fn unselected_capability_cannot_grant_commands() {
    let context: tauri::Context<tauri::Wry> = tauri::generate_context!();
    let extra = Capability {
        identifier: "unselected-file".into(),
        description: "Synthetic regression fixture; never selected".into(),
        local: true,
        remote: None,
        windows: vec!["main".into()],
        webviews: vec![],
        permissions: vec![PermissionEntry::PermissionRef(
            String::from("core:window:allow-set-title")
                .try_into()
                .unwrap(),
        )],
        platforms: None,
    };
    let files = BTreeMap::from([(extra.identifier.clone(), extra)]);
    let selected = get_capabilities(context.config(), files, None).unwrap();
    assert!(
        !selected.contains_key("unselected-file"),
        "An unselected capability file must not grant commands"
    );
    assert_eq!(selected.len(), 1);
    assert!(selected.values().all(|cap| cap.local
        && cap.remote.is_none()
        && cap.windows == ["main"]
        && cap.webviews.is_empty()
        && cap.permissions.is_empty()));
}

#[test]
fn compiled_authority_denies_ungranted_commands_and_origins() {
    let mut context: tauri::Context<tauri::Wry> = tauri::generate_context!();
    let authority = context.runtime_authority_mut();
    let origins = [
        Origin::Local,
        Origin::Remote {
            url: "https://example.invalid".parse().unwrap(),
        },
    ];
    for command in [
        "plugin:window|set_title",
        "plugin:event|emit",
        "plugin:shell|execute",
        "plugin:fs|write_text_file",
        "plugin:updater|check",
        "plugin:http|fetch",
    ] {
        for (window, webview) in [("main", "main"), ("other", "other"), ("main", "other")] {
            for origin in &origins {
                assert!(
                    authority
                        .resolve_access(command, window, webview, origin)
                        .is_none(),
                    "Unexpected grant: {command} / {window} / {webview}"
                );
            }
        }
    }
}

#[cfg(windows)]
mod native {
    use std::os::windows::fs::MetadataExt;
    use std::path::Path;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    };
    use std::time::Duration;
    use tauri::Listener;

    fn guard_profile(workspace: &Path, profile: &Path) {
        assert!(workspace.is_absolute() && profile.is_absolute());
        assert_eq!(profile, workspace.join("artifacts/sec001-webview-profile"));
        assert!(!profile.exists(), "Probe profile already exists");
        for ancestor in profile.ancestors() {
            match std::fs::symlink_metadata(ancestor) {
                Ok(metadata) => assert_eq!(
                    metadata.file_attributes() & 0x400,
                    0,
                    "Profile ancestor is a reparse point"
                ),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => panic!("Cannot validate profile ancestor: {error}"),
            }
        }
    }

    #[test]
    fn profile_guard_rejects_a_real_junction_without_writing_outside() {
        let project = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let fixture = project.join("artifacts/sec001-guard-fixture");
        assert!(
            !fixture.exists(),
            "Previous controlled fixture still exists"
        );
        let workspace = fixture.join("workspace");
        let profile = workspace.join("artifacts/sec001-webview-profile");
        guard_profile(&workspace, &profile);
        std::fs::create_dir_all(&workspace).unwrap();
        let target = fixture.join("target");
        std::fs::create_dir(&target).unwrap();
        let sentinel = target.join("sentinel");
        std::fs::write(&sentinel, b"unchanged").unwrap();
        let junction = workspace.join("artifacts");
        let literal = |path: &Path| format!("'{}'", path.display().to_string().replace('\'', "''"));
        let script = format!(
            "New-Item -ItemType Junction -Path {} -Target {} -ErrorAction Stop | Out-Null",
            literal(&junction),
            literal(&target)
        );
        let status = std::process::Command::new("rtk")
            .args([
                "proxy",
                "pwsh",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &script,
            ])
            .status()
            .unwrap();
        assert!(status.success());
        let denied = std::panic::catch_unwind(|| guard_profile(&workspace, &profile));
        assert!(denied.is_err());
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"unchanged");
        assert!(!target.join("sec001-webview-profile").exists());
        // Remove the junction itself, then only the owned non-recursive entries.
        std::fs::remove_dir(&junction).unwrap();
        std::fs::remove_file(&sentinel).unwrap();
        std::fs::remove_dir(&target).unwrap();
        std::fs::remove_dir(&workspace).unwrap();
        std::fs::remove_dir(&fixture).unwrap();
    }

    #[derive(Debug)]
    struct Outcome {
        brand_loaded: bool,
        title_denied: bool,
        event_denied: bool,
        title_unchanged: bool,
        event_unchanged: bool,
    }

    struct Probe {
        sender: mpsc::Sender<Outcome>,
        event_seen: Arc<AtomicBool>,
        initial_title: String,
    }

    // Test-only reporting endpoint. No invoke_handler exists in the product.
    #[tauri::command]
    fn security_probe_report(
        window: tauri::WebviewWindow,
        state: tauri::State<'_, Probe>,
        brand_loaded: bool,
        title_denied: bool,
        event_denied: bool,
    ) {
        state
            .sender
            .send(Outcome {
                brand_loaded,
                title_denied,
                event_denied,
                title_unchanged: window.title().unwrap() == state.initial_title,
                event_unchanged: !state.event_seen.load(Ordering::SeqCst),
            })
            .unwrap();
    }

    #[test]
    #[ignore = "Windows/WebView2, built frontend and tauri/custom-protocol required"]
    fn local_page_works_and_denied_ipc_has_no_effect() {
        let profile = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("artifacts/sec001-webview-profile");
        guard_profile(
            Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap(),
            &profile,
        );
        let mut context: tauri::Context<tauri::Wry> = tauri::generate_context!();
        let windows = std::mem::take(&mut context.config_mut().app.windows);
        assert_eq!(windows.len(), 1);
        let config = windows.into_iter().next().unwrap();
        assert_eq!(config.label, "main");
        let (sender, receiver) = mpsc::channel();
        let event_seen = Arc::new(AtomicBool::new(false));
        let listener_seen = event_seen.clone();
        let once = AtomicBool::new(false);
        let app = tauri::Builder::default()
            .any_thread()
            .manage(Probe {
                sender,
                event_seen,
                initial_title: config.title.clone(),
            })
            .invoke_handler(tauri::generate_handler![security_probe_report])
            .setup(move |app| {
                app.listen("sec-001-probe", move |_| {
                    listener_seen.store(true, Ordering::SeqCst);
                });
                tauri::WebviewWindowBuilder::from_config(app, &config)?
                    .visible(false)
                    .skip_taskbar(true)
                    .data_directory(profile)
                    .build()?;
                Ok(())
            })
            .on_page_load(move |webview, payload| {
                if payload.event() == tauri::webview::PageLoadEvent::Finished
                    && payload.url().host_str() == Some("tauri.localhost")
                    && !once.swap(true, Ordering::SeqCst)
                {
                    webview
                        .eval(
                            r#"(async () => {
                              await new Promise(resolve => setTimeout(resolve, 100));
                              const invoke = window.__TAURI_INTERNALS__.invoke;
                              const denied = async (command, args) => {
                                try { await invoke(command, args); return false; }
                                catch (error) { return /not allowed|not permitted/i.test(String(error)); }
                              };
                              const titleDenied = await denied("plugin:window|set_title",
                                {label: "main", value: "SEC-001 unauthorized"});
                              const eventDenied = await denied("plugin:event|emit",
                                {event: "sec-001-probe", payload: null});
                              await new Promise(resolve => setTimeout(resolve, 100));
                              await invoke("security_probe_report", {
                                brandLoaded: document.querySelector("h1")?.textContent === "Millani Artes",
                                titleDenied, eventDenied
                              });
                            })();"#,
                        )
                        .unwrap();
                }
            })
            .build(context)
            .unwrap();
        let handle = app.handle().clone();
        let result = std::thread::spawn(move || {
            let outcome = receiver.recv_timeout(Duration::from_secs(30));
            handle.exit(0);
            outcome
        });
        assert_eq!(app.run_return(|_, _| {}), 0);
        let outcome = result.join().unwrap().expect("Native IPC probe timed out");
        assert!(
            outcome.brand_loaded
                && outcome.title_denied
                && outcome.event_denied
                && outcome.title_unchanged
                && outcome.event_unchanged,
            "{outcome:?}"
        );
        println!("SEC001_NATIVE: {outcome:?}");
    }
}
