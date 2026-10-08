use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

const PHONE_TEST_NAME: &str = "startup_tests::phone_is_absent_from_startup_diagnostics";
const PHONE_FIXTURE: &str = "PHONE_SENTINEL_014 unknown={\"nested\":{\"phone\":\"+55 (11) 90000-0000\"}}\r\n(11) 90000-0000; 11900000000; １１９００００００００";
const FINANCIAL_TEST_NAME: &str =
    "startup_tests::financial_data_is_absent_from_startup_diagnostics";
const FINANCIAL_FIXTURE: &str = "FINANCE_SENTINEL_015 balanceCents=-765432109 priceCents=1234567 quantity=23 zeroCents=0 account=CONTA_SINTETICA_015 category=CATEGORIA_SINTETICA_015 context=Fábrica/Casa customerId=CLIENTE_SINTETICO_015 saleId=VENDA_SINTETICA_015 movementId=MOVIMENTO_SINTETICO_015 unknown={\"nested\":{\"totalCents\":987654321,\"display\":\"R$ 9.876.543,21\",\"aggregate\":\"-7.654.321,09\",\"decimal\":\"9876543.21\"}}\r\n＄１２３４５６７";
const STARTUP: &[u8] =
    b"{\"level\":\"error\",\"operation\":\"startup\",\"code\":\"TAURI_STARTUP_FAILED\"}\n";
const PANIC: &[u8] =
    b"{\"level\":\"error\",\"operation\":\"runtime\",\"code\":\"INTERNAL_PANIC\"}\n";

fn read_pipe(pipe: impl Read + Send + 'static) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        pipe.take(16_385)
            .read_to_end(&mut bytes)
            .expect("STARTUP_PIPE_READ_FAILED");
        bytes
    })
}

fn child(case: &str, fixture: &'static str) -> ! {
    std::io::stdin()
        .read_exact(&mut [0])
        .expect("STARTUP_BARRIER_FAILED");
    let injected = AtomicBool::new(false);
    let outcome = std::panic::catch_unwind(|| {
        super::run_startup(|| {
            if case == "panic" {
                std::panic::panic_any(fixture);
            }
            if case == "before-injection" {
                return Err(tauri::Error::Io(std::io::Error::other(
                    "CONTROLLED_BEFORE_INJECTION",
                )));
            }
            let mut context = tauri::generate_context!();
            context.config_mut().app.windows.clear();
            let _app = tauri::Builder::default().any_thread().build(context)?;
            if case == "ok" {
                Ok(())
            } else {
                injected.store(true, Ordering::Relaxed);
                Err(tauri::Error::Io(std::io::Error::other(fixture)))
            }
        })
    });
    if ["error", "closed", "before-injection"].contains(&case) && !injected.load(Ordering::Relaxed)
    {
        std::process::exit(102);
    }
    std::process::exit(match outcome {
        Ok(code) if code == std::process::ExitCode::SUCCESS => 0,
        Ok(_) => 1,
        Err(_) => 101,
    });
}

// ponytail: startup/hook coverage; add probes when financial/backup emitters exist.
fn check_diagnostics(test_name: &str, fixture: &'static str, markers: &[&str], leak_code: &str) {
    let args: Vec<_> = std::env::args().collect();
    let exact = args.windows(2).any(|pair| pair == ["--exact", test_name]);
    if exact {
        if let Ok(case) = std::env::var("MILLANI_STARTUP_CASE") {
            assert!(
                ["ok", "error", "panic", "closed", "before-injection"].contains(&case.as_str()),
                "STARTUP_CASE_INVALID"
            );
            child(&case, fixture);
        }
    }
    for (case, status, expected) in [
        ("ok", 0, &[][..]),
        ("error", 1, STARTUP),
        ("panic", 101, PANIC),
        ("closed", 1, &[][..]),
        ("before-injection", 102, STARTUP),
    ] {
        let mut child = Command::new(std::env::current_exe().expect("STARTUP_TEST_EXE_MISSING"))
            .args([
                "--quiet",
                "--exact",
                test_name,
                "--nocapture",
                "--color",
                "never",
            ])
            .env("MILLANI_STARTUP_CASE", case)
            .env("RUST_BACKTRACE", "full")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("STARTUP_CHILD_LAUNCH_FAILED");
        let stdout = read_pipe(child.stdout.take().expect("STARTUP_STDOUT_MISSING"));
        let stderr = child.stderr.take().expect("STARTUP_STDERR_MISSING");
        let stderr = if case == "closed" {
            drop(stderr);
            None
        } else {
            Some(read_pipe(stderr))
        };
        let result = (|| -> std::io::Result<()> {
            child
                .stdin
                .take()
                .ok_or_else(|| std::io::Error::other("STARTUP_STDIN_MISSING"))?
                .write_all(b"G")?;
            let deadline = Instant::now() + Duration::from_secs(20);
            while child.try_wait()?.is_none() {
                if Instant::now() >= deadline {
                    return Err(std::io::Error::other("STARTUP_CHILD_TIMEOUT"));
                }
                thread::sleep(Duration::from_millis(10));
            }
            Ok(())
        })();
        if result.is_err() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let exit = child.wait().expect("STARTUP_CHILD_WAIT_FAILED");
        let out = stdout.join().expect("STARTUP_STDOUT_THREAD_FAILED");
        let err = stderr
            .map(|reader| reader.join().expect("STARTUP_STDERR_THREAD_FAILED"))
            .unwrap_or_default();
        assert!(result.is_ok(), "STARTUP_CHILD_CONTROL_FAILED");
        assert!(
            out.len() <= 16_384 && err.len() <= 16_384,
            "STARTUP_OUTPUT_TOO_LARGE"
        );
        for &marker in markers {
            assert!(
                !String::from_utf8_lossy(&out).contains(marker)
                    && !String::from_utf8_lossy(&err).contains(marker),
                "{leak_code}"
            );
        }
        assert!(
            std::str::from_utf8(&out).map(str::trim) == Ok("running 1 test"),
            "STARTUP_STDOUT_NOT_STATIC"
        );
        assert!(exit.code() == Some(status), "STARTUP_EXIT_MISMATCH");
        assert!(err == expected, "STARTUP_DIAGNOSTIC_NOT_STATIC");
    }
}

#[test]
fn phone_is_absent_from_startup_diagnostics() {
    check_diagnostics(
        PHONE_TEST_NAME,
        PHONE_FIXTURE,
        &[
            "PHONE_SENTINEL_014",
            "+55 (11) 90000-0000",
            "(11) 90000-0000",
            "11900000000",
            "１１９００００００００",
            "0000",
        ],
        "STARTUP_PHONE_LEAK_DETECTED",
    );
    println!("PHONE_REDACTION_EVIDENCE {{\"cases\":5,\"realTauriBuild\":true,\"faultInjectedIo\":true,\"earlyFailureRejected\":true,\"panicHook\":true,\"closedStderr\":true,\"rawOutputPublished\":false,\"childrenTerminated\":true}}");
}

#[test]
fn financial_data_is_absent_from_startup_diagnostics() {
    check_diagnostics(
        FINANCIAL_TEST_NAME,
        FINANCIAL_FIXTURE,
        &[
            "FINANCE_SENTINEL_015",
            "765432109",
            "1234567",
            "23",
            "0",
            "CONTA_SINTETICA_015",
            "CATEGORIA_SINTETICA_015",
            "Fábrica",
            "Casa",
            "CLIENTE_SINTETICO_015",
            "VENDA_SINTETICA_015",
            "MOVIMENTO_SINTETICO_015",
            "987654321",
            "9.876.543,21",
            "7.654.321,09",
            "9876543.21",
            "１２３４５６７",
        ],
        "STARTUP_FINANCIAL_LEAK_DETECTED",
    );
    println!("FINANCIAL_REDACTION_EVIDENCE {{\"cases\":5,\"realTauriBuild\":true,\"faultInjectedIo\":true,\"earlyFailureRejected\":true,\"panicHook\":true,\"closedStderr\":true,\"rawOutputPublished\":false,\"childrenTerminated\":true,\"productionChanged\":false}}");
}
