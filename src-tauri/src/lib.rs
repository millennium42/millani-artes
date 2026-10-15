#[path = "infrastructure/database.rs"]
pub mod database;
pub mod domain;
pub mod application {
    pub mod create_account;
}
pub mod infrastructure {
    pub mod accounts;
}
