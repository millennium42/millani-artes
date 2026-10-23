#[path = "infrastructure/database.rs"]
pub mod database;
pub mod domain;
pub mod application {
    pub mod calculate_context_result;
    pub mod create_account;
    pub mod create_category;
    pub mod create_entry;
    pub mod create_exit;
    pub mod create_product;
    pub mod create_transfer;
    pub mod deactivate_category;
    pub mod deactivate_product;
    pub mod edit_movement;
    pub mod edit_product;
    pub mod generate_fixed_obligation;
    pub mod generate_variable_obligation;
    pub mod list_accounts;
    pub mod list_movements;
    pub mod mark_obligation_overdue;
    pub mod pay_recurrence;
    pub mod rename_category;
    pub mod supply_obligation_amount;
}
pub mod infrastructure {
    pub mod accounts;
    pub mod categories;
    pub mod movements;
    pub mod products;
    pub mod recurrences;
}
