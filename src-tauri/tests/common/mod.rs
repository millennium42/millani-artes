use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_DIRECTORY: AtomicU64 = AtomicU64::new(0);

pub fn open_clean_database() -> rusqlite::Result<Connection> {
    Connection::open_in_memory()
}

pub struct TempDirectory {
    root: PathBuf,
    path: PathBuf,
}

impl TempDirectory {
    pub fn new() -> Self {
        let root =
            std::fs::canonicalize(std::env::temp_dir()).expect("TEMP_DIRECTORY_ROOT_UNAVAILABLE");
        for _ in 0..128 {
            let id = NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = root.join(format!("millani-db002-{}-{id}", std::process::id()));
            match std::fs::create_dir(&path) {
                Ok(()) => {
                    if !std::fs::canonicalize(&path).is_ok_and(|canonical| canonical == path) {
                        panic!("TEMP_DIRECTORY_PATH_UNSAFE");
                    }
                    return Self { root, path };
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => panic!("TEMP_DIRECTORY_CREATE_FAILED"),
            }
        }
        panic!("TEMP_DIRECTORY_UNAVAILABLE")
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let safe_target = self.path.is_absolute()
            && self.path.parent() == Some(self.root.as_path())
            && std::fs::canonicalize(&self.root).is_ok_and(|canonical| canonical == self.root)
            && std::fs::canonicalize(&self.path).is_ok_and(|canonical| canonical == self.path);
        let safe_root = std::fs::symlink_metadata(&self.root)
            .is_ok_and(|metadata| metadata.is_dir() && !has_reparse_point(&metadata));
        let safe_directory = std::fs::symlink_metadata(&self.path)
            .is_ok_and(|metadata| metadata.is_dir() && !has_reparse_point(&metadata));
        if safe_target && safe_root && safe_directory {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

fn has_reparse_point(metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_type().is_symlink() || metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

mod movements_migration;

mod movements_constraints;

mod entry_failures;
mod entry_persistence;

mod category_creation;

mod category_rename;

mod category_deactivation;

mod category_history;
