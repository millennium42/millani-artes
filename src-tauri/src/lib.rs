#[path = "infrastructure/database.rs"]
pub mod database;
pub mod domain;
pub mod application {
    pub mod create_account;
    pub mod create_category;
    pub mod create_entry;
    pub mod create_exit;
    pub mod create_transfer;
    pub mod deactivate_category;
    pub mod list_accounts;
    pub mod list_movements;
    pub mod rename_category;
}
pub mod infrastructure {
    pub mod accounts;
    pub mod categories;
    pub mod movements;
}
