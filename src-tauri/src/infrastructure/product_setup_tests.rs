use super::{create_product_from_request_for, initialize_product_store_at, ProductSetupState};
use crate::infrastructure::AppDatabaseWriteLock;
use millani_artes::database;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "millani-ui008-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        assert!(!path.exists(), "UI008_TEST_DIRECTORY_ALREADY_EXISTS");
        std::fs::create_dir(&path).expect("UI008_TEST_DIRECTORY_CREATE_FAILED");
        Self(path)
    }

    fn data_path(&self) -> PathBuf {
        self.0.join("app-data")
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn setup() -> (TestDirectory, PathBuf, ProductSetupState) {
    let directory = TestDirectory::new();
    let data_path = directory.data_path();
    let store = initialize_product_store_at(&data_path).expect("UI008_DB_INIT_FAILED");
    let state = ProductSetupState::new(store, data_path.clone());
    (directory, data_path, state)
}

fn body(name: &str, description: &str, cents: &str) -> tauri::ipc::InvokeBody {
    let mut body = tauri::ipc::InvokeBody::Json(Default::default());
    let tauri::ipc::InvokeBody::Json(arguments) = &mut body else {
        unreachable!("UI008_JSON_BODY_EXPECTED");
    };
    arguments["name"] = name.into();
    arguments["description"] = description.into();
    arguments["suggestedPriceCents"] = cents.into();
    body
}

fn counts(path: &Path) -> (i64, i64, i64, i64, i64, i64) {
    let connection = database::open_database(&path.join("millani-artes.sqlite3"))
        .expect("UI008_DATABASE_REOPEN_FAILED");
    let count = |table: &str| {
        connection
            .query_row(&format!("SELECT COUNT(*) FROM main.{table}"), [], |row| {
                row.get(0)
            })
            .expect("UI008_TABLE_COUNT_FAILED")
    };
    let products = count("products");
    let audits = count("audit_event");
    let accounts = count("accounts");
    let movements = count("movements");
    let sales = count("sales");
    let receivables = count("receivables");
    (products, audits, accounts, movements, sales, receivables)
}

#[test]
fn creates_active_product_and_audit_without_financial_or_inventory_effects() {
    let (_directory, data_path, state) = setup();

    create_product_from_request_for(
        &state,
        &AppDatabaseWriteLock::default(),
        &body("  Modelo Árvore 🪴  ", "  Feito à mão  ", "0"),
    )
    .expect("UI008_CREATE_FAILED");

    let connection = database::open_database(&data_path.join("millani-artes.sqlite3"))
        .expect("UI008_DATABASE_REOPEN_FAILED");
    let (name, description, price, is_active): (String, String, i64, bool) = connection
        .query_row(
            "SELECT name, description, suggested_price_cents, is_active FROM main.products",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("UI008_PRODUCT_READ_FAILED");
    assert_eq!(name, "  Modelo Árvore 🪴  ");
    assert_eq!(description, "  Feito à mão  ");
    assert_eq!(price, 0);
    assert!(is_active);

    let (operation, entity, entity_id_length): (String, String, i64) = connection
        .query_row(
            "SELECT operation, entity, length(entity_id) FROM main.audit_event",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("UI008_AUDIT_READ_FAILED");
    assert_eq!(operation, "product_create");
    assert_eq!(entity, "product");
    assert_eq!(entity_id_length, 16);

    let tables: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM main.sqlite_schema WHERE type='table' AND name IN ('inventory', 'inventories', 'stock', 'stocks', 'product_stock')",
            [],
            |row| row.get(0),
        )
        .expect("UI008_INVENTORY_SCHEMA_CHECK_FAILED");
    assert_eq!(tables, 0);
    assert_eq!(counts(&data_path), (1, 1, 0, 0, 0, 0));
}

#[cfg(target_os = "windows")]
#[test]
#[ignore = "Requires Windows WebView2 and production assets; hidden functional test, no visual inspection"]
fn native_ui_submit_creates_product_through_ipc_and_sqlite() {
    let (directory, data_path, state) = setup();
    let profile_path = directory.0.join("webview-profile");
    assert!(
        !profile_path.exists(),
        "UI008_WEBVIEW_PROFILE_ALREADY_EXISTS"
    );

    let mut context: tauri::Context<tauri::Wry> = tauri::generate_context!();
    let mut windows = std::mem::take(&mut context.config_mut().app.windows);
    assert_eq!(windows.len(), 1, "UI008_MAIN_WINDOW_CONFIG_MISSING");
    let window_config = windows.remove(0);
    let inject_once = Arc::new(AtomicBool::new(false));
    let page_inject_once = Arc::clone(&inject_once);

    let app = tauri::Builder::default()
        .any_thread()
        .manage(state)
        .manage(AppDatabaseWriteLock::default())
        .invoke_handler(tauri::generate_handler![super::create_product])
        .setup(move |app| {
            tauri::WebviewWindowBuilder::from_config(app, &window_config)?
                .visible(false)
                .skip_taskbar(true)
                .data_directory(profile_path)
                .on_page_load(move |webview, payload| {
                    if payload.event() == tauri::webview::PageLoadEvent::Finished
                        && !page_inject_once.swap(true, Ordering::SeqCst)
                    {
                        webview
                            .eval(
                                r#"(async () => {
                                  const deadline = Date.now() + 10000;
                                  let name = document.querySelector('#product-name');
                                  while (!name && Date.now() < deadline) {
                                    await new Promise(resolve => setTimeout(resolve, 25));
                                    name = document.querySelector('#product-name');
                                  }
                                  const description = document.querySelector('#product-description');
                                  const price = document.querySelector('#product-price');
                                  if (!name || !description || !price) return;
                                  const setValue = (element, prototype, value) => {
                                    Object.getOwnPropertyDescriptor(prototype, 'value').set.call(element, value);
                                    element.dispatchEvent(new Event('input', { bubbles: true }));
                                  };
                                  setValue(name, HTMLInputElement.prototype, 'UI008 native synthetic');
                                  setValue(description, HTMLTextAreaElement.prototype, 'Temporary IPC data');
                                  setValue(price, HTMLInputElement.prototype, '12,34');
                                  await new Promise(resolve => setTimeout(resolve, 50));
                                  name.form.requestSubmit();
                                })();"#,
                            )
                            .expect("UI008_NATIVE_FORM_INJECTION_FAILED");
                    }
                })
                .build()?;
            Ok(())
        })
        .build(context)
        .expect("UI008_NATIVE_TAURI_BUILD_FAILED");

    let app_handle = app.handle().clone();
    let observed_data_path = data_path.clone();
    let (finished_tx, finished_rx) = mpsc::channel();
    let monitor = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let product_exists =
                database::open_database(&observed_data_path.join("millani-artes.sqlite3"))
                    .ok()
                    .and_then(|connection| {
                        connection
                            .query_row(
                                "SELECT COUNT(*) FROM main.products WHERE name = ?1",
                                ["UI008 native synthetic"],
                                |row| row.get::<_, i64>(0),
                            )
                            .ok()
                    })
                    .unwrap_or_default()
                    == 1;
            if product_exists {
                let _ = finished_tx.send(true);
                app_handle.exit(0);
                return;
            }
            if Instant::now() >= deadline {
                let _ = finished_tx.send(false);
                app_handle.exit(1);
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    });

    let exit_code = app.run_return(|_, _| {});
    let created = finished_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("UI008_NATIVE_MONITOR_TIMEOUT");
    monitor.join().expect("UI008_NATIVE_MONITOR_PANICKED");
    assert_eq!(exit_code, 0, "UI008_NATIVE_APP_EXIT_FAILED");
    assert!(created, "UI008_NATIVE_PRODUCT_SUBMISSION_FAILED");

    let connection = database::open_database(&data_path.join("millani-artes.sqlite3"))
        .expect("UI008_DATABASE_REOPEN_FAILED");
    let (name, description, price): (String, String, i64) = connection
        .query_row(
            "SELECT name, description, suggested_price_cents FROM main.products",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("UI008_PRODUCT_READ_FAILED");
    assert_eq!(name, "UI008 native synthetic");
    assert_eq!(description, "Temporary IPC data");
    assert_eq!(price, 1234);
    assert_eq!(counts(&data_path), (1, 1, 0, 0, 0, 0));
}

#[test]
fn persists_an_empty_description() {
    let (_directory, data_path, state) = setup();
    create_product_from_request_for(
        &state,
        &AppDatabaseWriteLock::default(),
        &body("Produto sem descrição", "", "0"),
    )
    .expect("UI008_EMPTY_DESCRIPTION_CREATE_FAILED");

    let connection = database::open_database(&data_path.join("millani-artes.sqlite3"))
        .expect("UI008_DATABASE_REOPEN_FAILED");
    let description: String = connection
        .query_row("SELECT description FROM main.products", [], |row| {
            row.get(0)
        })
        .expect("UI008_EMPTY_DESCRIPTION_READ_FAILED");
    assert_eq!(description, "");
    assert_eq!(counts(&data_path), (1, 1, 0, 0, 0, 0));
}

#[test]
fn rejects_unicode_whitespace_name_before_writing() {
    let (_directory, data_path, state) = setup();

    assert_eq!(
        create_product_from_request_for(
            &state,
            &AppDatabaseWriteLock::default(),
            &body(" \u{00a0}\u{2007}\u{202f}\u{3000} ", "descrição", "0"),
        ),
        Err("PRODUCT_NAME_INVALID")
    );
    assert_eq!(counts(&data_path), (0, 0, 0, 0, 0, 0));
}

#[test]
fn rejects_broad_malformed_and_noncanonical_payloads_without_writes() {
    let (_directory, data_path, state) = setup();
    let write_lock = AppDatabaseWriteLock::default();

    assert_eq!(
        create_product_from_request_for(
            &state,
            &write_lock,
            &tauri::ipc::InvokeBody::Raw(b"not-json".to_vec()),
        ),
        Err("PRODUCT_INVALID")
    );

    let mut extra = body("Produto", "", "0");
    if let tauri::ipc::InvokeBody::Json(arguments) = &mut extra {
        arguments["unexpected"] = "secret".into();
    }
    assert_eq!(
        create_product_from_request_for(&state, &write_lock, &extra),
        Err("PRODUCT_INVALID")
    );

    let mut missing = body("Produto", "", "0");
    if let tauri::ipc::InvokeBody::Json(arguments) = &mut missing {
        arguments.as_object_mut().unwrap().remove("description");
    }
    assert_eq!(
        create_product_from_request_for(&state, &write_lock, &missing),
        Err("PRODUCT_INVALID")
    );

    let mut wrong_type = body("Produto", "", "0");
    if let tauri::ipc::InvokeBody::Json(arguments) = &mut wrong_type {
        arguments["name"] = 123.into();
    }
    assert_eq!(
        create_product_from_request_for(&state, &write_lock, &wrong_type),
        Err("PRODUCT_INVALID")
    );

    for price in ["", "-1", "00", "+1", "1.0", "9223372036854775808"] {
        assert_eq!(
            create_product_from_request_for(&state, &write_lock, &body("Produto", "", price)),
            Err("PRODUCT_PRICE_INVALID"),
            "UI008_NONCANONICAL_PRICE_REJECTED"
        );
    }
    assert_eq!(counts(&data_path), (0, 0, 0, 0, 0, 0));
}

#[test]
fn audit_failure_rolls_back_recovers_store_and_allows_retry() {
    let (_directory, data_path, state) = setup();
    let database_path = data_path.join("millani-artes.sqlite3");
    let connection = database::open_database(&database_path).expect("UI008_DB_OPEN_FAILED");
    connection
        .execute_batch(
            "CREATE TRIGGER main.reject_ui008_audit
             BEFORE INSERT ON audit_event
             WHEN NEW.operation = 'product_create'
             BEGIN SELECT RAISE(ABORT, 'synthetic UI008 persistence failure'); END;",
        )
        .expect("UI008_TRIGGER_CREATE_FAILED");
    drop(connection);

    let write_lock = AppDatabaseWriteLock::default();
    let request = body("Retry produto", "Descrição", "1234");
    assert_eq!(
        create_product_from_request_for(&state, &write_lock, &request),
        Err("PRODUCT_CREATE_FAILED")
    );
    assert_eq!(counts(&data_path), (0, 0, 0, 0, 0, 0));

    let connection = database::open_database(&database_path).expect("UI008_DB_REOPEN_FAILED");
    connection
        .execute_batch("DROP TRIGGER main.reject_ui008_audit;")
        .expect("UI008_TRIGGER_DROP_FAILED");
    drop(connection);

    create_product_from_request_for(&state, &write_lock, &request).expect("UI008_RETRY_FAILED");
    assert_eq!(counts(&data_path), (1, 1, 0, 0, 0, 0));
}

#[test]
fn poisoned_shared_write_lock_returns_static_error_without_writing() {
    let (_directory, data_path, state) = setup();
    let write_lock = AppDatabaseWriteLock::default();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = write_lock.lock().expect("UI008_LOCK_ACQUIRE_FAILED");
        panic!("synthetic UI008 lock poisoning");
    }));
    assert!(panic.is_err());

    assert_eq!(
        create_product_from_request_for(&state, &write_lock, &body("Produto", "", "0")),
        Err("PRODUCT_CREATE_FAILED")
    );
    assert_eq!(counts(&data_path), (0, 0, 0, 0, 0, 0));
}
