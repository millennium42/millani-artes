use super::{initialize_category_store_at, CategorySetupState};
use millani_artes::database;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "millani-ui004-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        assert!(!path.exists(), "UI004_TEST_DIRECTORY_ALREADY_EXISTS");
        std::fs::create_dir(&path).expect("UI004_TEST_DIRECTORY_CREATE_FAILED");
        Self(path)
    }

    fn data_path(&self) -> PathBuf {
        self.0.join("app-data")
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("UI004_TEST_DIRECTORY_CLEANUP_FAILED");
    }
}

#[test]
fn creates_a_category_with_audit_and_preserves_the_literal_name() {
    let temp = TestDirectory::new();
    let data_path = temp.data_path();
    let store = initialize_category_store_at(&data_path).expect("UI004_STARTUP_FAILED");
    let state = CategorySetupState::new(store, data_path.clone());

    state
        .create_category("  Acessórios 🪴  ".to_owned())
        .expect("UI004_CATEGORY_CREATE_FAILED");

    let connection = database::open_database(&data_path.join("millani-artes.sqlite3"))
        .expect("UI004_DATABASE_REOPEN_FAILED");
    let (name, is_active): (String, bool) = connection
        .query_row("SELECT name, is_active FROM main.categories", [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .expect("UI004_CATEGORY_READ_FAILED");
    assert_eq!(name, "  Acessórios 🪴  ");
    assert!(is_active);

    let (operation, entity, entity_id_length): (String, String, i64) = connection
        .query_row(
            "SELECT operation, entity, length(entity_id) FROM main.audit_event",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("UI004_AUDIT_READ_FAILED");
    assert_eq!(operation, "category_create");
    assert_eq!(entity, "category");
    assert_eq!(entity_id_length, 16);
}

#[test]
fn rejects_unicode_whitespace_without_database_effects() {
    let temp = TestDirectory::new();
    let data_path = temp.data_path();
    let store = initialize_category_store_at(&data_path).expect("UI004_STARTUP_FAILED");
    let state = CategorySetupState::new(store, data_path.clone());

    assert_eq!(
        state.create_category(" \u{00a0}\u{2007}\u{202f}\u{3000} ".to_owned()),
        Err("CATEGORY_NAME_INVALID")
    );

    let connection = database::open_database(&data_path.join("millani-artes.sqlite3"))
        .expect("UI004_DATABASE_REOPEN_FAILED");
    let category_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM main.categories", [], |row| row.get(0))
        .expect("UI004_CATEGORY_COUNT_FAILED");
    let audit_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM main.audit_event", [], |row| {
            row.get(0)
        })
        .expect("UI004_AUDIT_COUNT_FAILED");
    assert_eq!(category_count, 0);
    assert_eq!(audit_count, 0);
}

#[test]
fn failed_category_write_rolls_back_recovers_and_allows_retry() {
    let temp = TestDirectory::new();
    let data_path = temp.data_path();
    let store = initialize_category_store_at(&data_path).expect("UI004_STARTUP_FAILED");
    let state = CategorySetupState::new(store, data_path.clone());
    let database_path = data_path.join("millani-artes.sqlite3");

    let connection = database::open_database(&database_path).expect("UI004_DATABASE_OPEN_FAILED");
    connection
        .execute_batch(
            "CREATE TRIGGER main.reject_ui004_audit
             BEFORE INSERT ON audit_event
             BEGIN SELECT RAISE(ABORT, 'synthetic UI004 persistence failure'); END;",
        )
        .expect("UI004_TRIGGER_CREATE_FAILED");
    drop(connection);

    assert_eq!(
        state.create_category("Categoria de teste".to_owned()),
        Err("CATEGORY_CREATE_FAILED")
    );

    let connection = database::open_database(&database_path).expect("UI004_DATABASE_REOPEN_FAILED");
    let category_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM main.categories", [], |row| row.get(0))
        .expect("UI004_CATEGORY_COUNT_FAILED");
    let audit_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM main.audit_event", [], |row| {
            row.get(0)
        })
        .expect("UI004_AUDIT_COUNT_FAILED");
    assert_eq!(category_count, 0);
    assert_eq!(audit_count, 0);
    connection
        .execute_batch("DROP TRIGGER main.reject_ui004_audit;")
        .expect("UI004_TRIGGER_DROP_FAILED");
    drop(connection);

    state
        .create_category("Categoria de teste".to_owned())
        .expect("UI004_RETRY_FAILED");

    let connection = database::open_database(&database_path).expect("UI004_DATABASE_REOPEN_FAILED");
    let category_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM main.categories", [], |row| row.get(0))
        .expect("UI004_CATEGORY_COUNT_FAILED");
    let audit_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM main.audit_event", [], |row| {
            row.get(0)
        })
        .expect("UI004_AUDIT_COUNT_FAILED");
    assert_eq!(category_count, 1);
    assert_eq!(audit_count, 1);
}
