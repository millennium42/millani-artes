#[path = "infrastructure/database.rs"]
pub mod database;
pub mod domain;
pub mod application {
    pub mod create_account;
    pub mod create_entry;
    pub mod list_accounts;
}
pub mod infrastructure {
    pub mod accounts;
    pub mod movements;
}
