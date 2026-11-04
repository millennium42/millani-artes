use crate::domain::{Receivable, SaleItem, SalePending};
use rusqlite::{params, OptionalExtension};
use rusqlite::{Connection, OpenFlags, Transaction, TransactionBehavior};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::path::Path;

#[derive(PartialEq, Eq)]
pub enum TransactionError<E> {
    BeginFailed,
    OperationFailed(E),
    CommitFailed,
    RollbackFailed,
}

impl<E> fmt::Debug for TransactionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::BeginFailed => "BeginFailed",
            Self::OperationFailed(_) => "OperationFailed",
            Self::CommitFailed => "CommitFailed",
            Self::RollbackFailed => "RollbackFailed",
        })
    }
}

impl<E> fmt::Display for TransactionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::BeginFailed => "write transaction could not begin",
            Self::OperationFailed(_) => "write transaction operation failed",
            Self::CommitFailed => "write transaction could not commit",
            Self::RollbackFailed => "write transaction rollback failed",
        })
    }
}

impl<E> Error for TransactionError<E> {}

/// Runs one write unit. The trusted callback must not control transactions, attach databases,
/// vacuum, or perform external effects; this API does not promise recovery after commit failure.
pub fn execute_write_transaction<T, E>(
    connection: Connection,
    operation: impl FnOnce(&Transaction<'_>) -> Result<T, E>,
) -> Result<(Connection, T), TransactionError<E>> {
    execute_transaction_with_behavior(connection, TransactionBehavior::Immediate, operation)
}

#[derive(Debug, PartialEq, Eq)]
pub enum DatabaseOpenError {
    InvalidPath,
    OpenFailed,
}

impl fmt::Display for DatabaseOpenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidPath => "database path is invalid",
            Self::OpenFailed => "database could not be opened",
        })
    }
}

impl Error for DatabaseOpenError {}

#[derive(Debug, PartialEq, Eq)]
pub enum MigrationError {
    BeginFailed,
    ApplyFailed,
    CommitFailed,
    RollbackFailed,
    InvalidVersion,
    InvalidSchemaState,
}

impl fmt::Display for MigrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::BeginFailed => "migration could not begin",
            Self::ApplyFailed => "migration could not be applied",
            Self::CommitFailed => "migration could not commit",
            Self::RollbackFailed => "migration rollback failed",
            Self::InvalidVersion => "migration version is invalid",
            Self::InvalidSchemaState => "database schema state is invalid",
        })
    }
}

impl Error for MigrationError {}

pub fn open_database(path: &Path) -> Result<Connection, DatabaseOpenError> {
    // shortcut: caller owns the parent directory; enforce app-data containment before accepting UI paths.
    if !path.is_absolute() {
        return Err(DatabaseOpenError::InvalidPath);
    }
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    )
    .map_err(|_| DatabaseOpenError::OpenFailed)?;
    connection
        .query_row("SELECT count(*) FROM sqlite_schema", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|_| DatabaseOpenError::OpenFailed)?;
    connection
        .pragma_update(None, "foreign_keys", true)
        .map_err(|_| DatabaseOpenError::OpenFailed)?;
    let foreign_keys: i64 = connection
        .pragma_query_value(None, "foreign_keys", |row| row.get(0))
        .map_err(|_| DatabaseOpenError::OpenFailed)?;
    if foreign_keys != 1 {
        return Err(DatabaseOpenError::OpenFailed);
    }
    Ok(connection)
}

/// Applies reviewed internal SQL without transaction control, ATTACH/DETACH, VACUUM, or external effects.
/// shortcut: static lifetime does not validate SQL; add a parser only if callers admit external scripts.
pub fn execute_migration(
    connection: Connection,
    sql: &'static str,
) -> Result<Connection, MigrationError> {
    execute_transaction(connection, |transaction| {
        transaction
            .execute_batch(sql)
            .map_err(|_| MigrationError::ApplyFailed)
    })
}

fn execute_transaction(
    connection: Connection,
    apply: impl FnOnce(&Transaction<'_>) -> Result<(), MigrationError>,
) -> Result<Connection, MigrationError> {
    execute_transaction_with_behavior(connection, TransactionBehavior::Deferred, apply)
        .map(|(connection, ())| connection)
        .map_err(|error| match error {
            TransactionError::BeginFailed => MigrationError::BeginFailed,
            TransactionError::OperationFailed(error) => error,
            TransactionError::CommitFailed => MigrationError::CommitFailed,
            TransactionError::RollbackFailed => MigrationError::RollbackFailed,
        })
}

fn execute_transaction_with_behavior<T, E>(
    mut connection: Connection,
    behavior: TransactionBehavior,
    operation: impl FnOnce(&Transaction<'_>) -> Result<T, E>,
) -> Result<(Connection, T), TransactionError<E>> {
    let transaction = connection
        .transaction_with_behavior(behavior)
        .map_err(|_| TransactionError::BeginFailed)?;
    let value = match operation(&transaction) {
        Ok(value) => value,
        Err(error) => {
            return match transaction.rollback() {
                Ok(()) => Err(TransactionError::OperationFailed(error)),
                Err(_) => Err(TransactionError::RollbackFailed),
            };
        }
    };
    transaction
        .commit()
        .map_err(|_| TransactionError::CommitFailed)?;
    Ok((connection, value))
}

/// Reads the main database's contiguous migration ledger up to the caller's compiled version.
pub fn schema_version(
    connection: &Connection,
    supported_version: u32,
) -> Result<u32, MigrationError> {
    let object_type = connection
        .query_row(
            "SELECT type FROM main.sqlite_schema WHERE name = 'schema_migration' AND type IN ('table', 'view')",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let Some(object_type) = object_type else {
        return if has_other_user_objects(connection)? {
            Err(MigrationError::InvalidSchemaState)
        } else {
            Ok(0)
        };
    };
    if object_type != "table" {
        return Err(MigrationError::InvalidSchemaState);
    }

    let mut columns = connection
        .prepare("PRAGMA main.table_xinfo('schema_migration')")
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let mut column_rows = columns
        .query([])
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let column = column_rows
        .next()
        .map_err(|_| MigrationError::InvalidSchemaState)?
        .ok_or(MigrationError::InvalidSchemaState)?;
    let name = column
        .get::<_, String>(1)
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let declared_type = column
        .get::<_, String>(2)
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let not_null = column
        .get::<_, i64>(3)
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let primary_key = column
        .get::<_, i64>(5)
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let hidden = column
        .get::<_, i64>(6)
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    if name != "version"
        || !declared_type.eq_ignore_ascii_case("INTEGER")
        || not_null != 1
        || primary_key != 1
        || hidden != 0
        || column_rows
            .next()
            .map_err(|_| MigrationError::InvalidSchemaState)?
            .is_some()
    {
        return Err(MigrationError::InvalidSchemaState);
    }
    drop(column_rows);
    drop(columns);

    let mut statement = connection
        .prepare("SELECT version, typeof(version) FROM main.schema_migration ORDER BY version")
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let mut rows = statement
        .query([])
        .map_err(|_| MigrationError::InvalidSchemaState)?;
    let mut previous = None;
    while let Some(row) = rows
        .next()
        .map_err(|_| MigrationError::InvalidSchemaState)?
    {
        let storage_class = row
            .get::<_, String>(1)
            .map_err(|_| MigrationError::InvalidSchemaState)?;
        if storage_class != "integer" {
            return Err(MigrationError::InvalidSchemaState);
        }
        let raw_version = row
            .get::<_, i64>(0)
            .map_err(|_| MigrationError::InvalidSchemaState)?;
        let version = u32::try_from(raw_version).map_err(|_| MigrationError::InvalidSchemaState)?;
        if version == 0
            || version > supported_version
            || previous.map_or(version != 1, |last: u32| {
                last.checked_add(1) != Some(version)
            })
        {
            return Err(MigrationError::InvalidSchemaState);
        }
        previous = Some(version);
    }
    drop(rows);
    drop(statement);

    let Some(current) = previous else {
        return if has_other_user_objects(connection)? {
            Err(MigrationError::InvalidSchemaState)
        } else {
            Ok(0)
        };
    };
    Ok(current)
}

/// Applies trusted, reviewed SQL for exactly the next version and records it atomically.
pub fn execute_versioned_migration(
    connection: Connection,
    version: u32,
    sql: &'static str,
) -> Result<Connection, MigrationError> {
    execute_versioned_migration_with(connection, version, sql, |_| Ok(()))
}

fn execute_versioned_migration_with(
    connection: Connection,
    version: u32,
    sql: &'static str,
    after_sql: impl FnOnce(&Transaction<'_>) -> Result<(), MigrationError>,
) -> Result<Connection, MigrationError> {
    if version == 0 {
        return Err(MigrationError::InvalidVersion);
    }
    execute_transaction(connection, |transaction| {
        let current = schema_version(transaction, version)?;
        if current.checked_add(1) != Some(version) {
            return Err(MigrationError::InvalidVersion);
        }
        transaction
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS main.schema_migration (version INTEGER PRIMARY KEY CHECK (version > 0) NOT NULL)",
            )
            .map_err(|_| MigrationError::ApplyFailed)?;
        transaction
            .execute_batch(sql)
            .map_err(|_| MigrationError::ApplyFailed)?;
        after_sql(transaction)?;
        transaction
            .execute(
                "INSERT INTO main.schema_migration(version) VALUES (?1)",
                [version],
            )
            .map_err(|_| MigrationError::ApplyFailed)?;
        Ok(())
    })
}

fn has_other_user_objects(connection: &Connection) -> Result<bool, MigrationError> {
    connection
        .query_row(
            "SELECT EXISTS (SELECT 1 FROM main.sqlite_schema WHERE name NOT GLOB 'sqlite_*' AND NOT (type = 'table' AND name = 'schema_migration'))",
            [],
            |row| row.get(0),
        )
        .map_err(|_| MigrationError::InvalidSchemaState)
}

pub const AUDIT_EVENT_MIGRATION_VERSION: u32 = 2;
pub const AUDIT_EVENT_MIGRATION_SQL: &str = include_str!("../../migrations/002-audit-event.sql");
pub const ACCOUNTS_MIGRATION_VERSION: u32 = 3;
pub const ACCOUNTS_MIGRATION_SQL: &str = include_str!("../../migrations/003-accounts.sql");
pub const CATEGORIES_MIGRATION_VERSION: u32 = 4;
pub const CATEGORIES_MIGRATION_SQL: &str = include_str!("../../migrations/004-categories.sql");
pub const MOVEMENTS_MIGRATION_VERSION: u32 = 5;
pub const MOVEMENTS_MIGRATION_SQL: &str = include_str!("../../migrations/005-movements.sql");
pub const MOVEMENT_CATEGORIES_MIGRATION_VERSION: u32 = 6;
pub const MOVEMENT_CATEGORIES_MIGRATION_SQL: &str =
    include_str!("../../migrations/006-movement-categories.sql");

pub const RECURRENCES_MIGRATION_VERSION: u32 = 7;
pub const RECURRENCES_MIGRATION_SQL: &str = include_str!("../../migrations/007-recurrences.sql");

pub const PRODUCTS_MIGRATION_VERSION: u32 = 8;
pub const PRODUCTS_MIGRATION_SQL: &str = include_str!("../../migrations/008-products.sql");

pub const CUSTOMERS_MIGRATION_VERSION: u32 = 9;
pub const CUSTOMERS_MIGRATION_SQL: &str = include_str!("../../migrations/009-customers.sql");

pub const SALES_MIGRATION_VERSION: u32 = 10;
pub const SALES_MIGRATION_SQL: &str = include_str!("../../migrations/010-sales.sql");

pub const SALE_ITEMS_MIGRATION_VERSION: u32 = 11;
pub const SALE_ITEMS_MIGRATION_SQL: &str = include_str!("../../migrations/011-sale-items.sql");

pub const RECEIVABLES_MIGRATION_VERSION: u32 = 12;
pub const RECEIVABLES_MIGRATION_SQL: &str = include_str!("../../migrations/012-receivables.sql");
pub const RECEIPTS_MIGRATION_VERSION: u32 = 13;
pub const RECEIPTS_MIGRATION_SQL: &str = include_str!("../../migrations/013-receipts.sql");
pub const SALE_RETURN_ITEMS_MIGRATION_VERSION: u32 = 14;
pub const SALE_RETURN_ITEMS_MIGRATION_SQL: &str =
    include_str!("../../migrations/014-sale-return-items.sql");
pub const SALE_RETURN_FINANCIALS_MIGRATION_VERSION: u32 = 15;
pub const SALE_RETURN_FINANCIALS_MIGRATION_SQL: &str =
    include_str!("../../migrations/015-sale-return-financials.sql");

pub fn execute_sale_return_financials_migration(
    connection: Connection,
) -> Result<Connection, MigrationError> {
    execute_versioned_migration_with(
        connection,
        SALE_RETURN_FINANCIALS_MIGRATION_VERSION,
        SALE_RETURN_FINANCIALS_MIGRATION_SQL,
        backfill_sale_return_financials,
    )
}

pub const SALE_RETURN_REFUNDS_MIGRATION_VERSION: u32 = 16;
pub const SALE_RETURN_REFUNDS_MIGRATION_SQL: &str =
    include_str!("../../migrations/016-sale-return-refunds.sql");

pub fn execute_sale_return_refunds_migration(
    connection: Connection,
) -> Result<Connection, MigrationError> {
    execute_versioned_migration(
        connection,
        SALE_RETURN_REFUNDS_MIGRATION_VERSION,
        SALE_RETURN_REFUNDS_MIGRATION_SQL,
    )
}

pub const SALE_CANCELLATIONS_MIGRATION_VERSION: u32 = 17;
pub const SALE_CANCELLATIONS_MIGRATION_SQL: &str =
    include_str!("../../migrations/017-sale-cancellations.sql");

pub fn execute_sale_cancellations_migration(
    connection: Connection,
) -> Result<Connection, MigrationError> {
    execute_versioned_migration(
        connection,
        SALE_CANCELLATIONS_MIGRATION_VERSION,
        SALE_CANCELLATIONS_MIGRATION_SQL,
    )
}

fn backfill_sale_return_financials(transaction: &Transaction<'_>) -> Result<(), MigrationError> {
    let receivables = {
        let mut statement = transaction
            .prepare(
                "SELECT r.id,r.sale_id,r.amount_cents,r.customer_id,r.due_at_ms,COALESCE(SUM(receipt.amount_cents),0) FROM main.receivables AS r LEFT JOIN main.receipts AS receipt ON receipt.receivable_id=r.id GROUP BY r.id,r.sale_id,r.amount_cents,r.customer_id,r.due_at_ms ORDER BY r.rowid",
            )
            .map_err(|_| MigrationError::ApplyFailed)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            })
            .map_err(|_| MigrationError::ApplyFailed)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| MigrationError::ApplyFailed)?;
        rows
    };
    let mut due_by_sale = HashMap::with_capacity(receivables.len());
    for (id, sale_id, principal_cents, customer_id, due_at_ms, receipt_cents) in receivables {
        let sale_id = migration_id(sale_id)?;
        let receivable = Receivable {
            id: migration_id(id)?,
            sale_id,
            pending: SalePending {
                amount_cents: principal_cents,
                customer_id: migration_id(customer_id)?,
                due_at_ms,
            },
        };
        let due_cents = receivable
            .remaining_due_cents(receipt_cents)
            .map_err(|_| MigrationError::ApplyFailed)?;
        if due_by_sale.insert(sale_id, due_cents).is_some() {
            return Err(MigrationError::ApplyFailed);
        }
    }

    let return_count: i64 = transaction
        .query_row("SELECT COUNT(*) FROM main.sale_return_items", [], |row| {
            row.get(0)
        })
        .map_err(|_| MigrationError::ApplyFailed)?;
    let returns = {
        let mut statement = transaction
            .prepare(
                "SELECT returned.id,returned.sale_item_id,item.sale_id,returned.quantity,item.quantity,item.total_cents FROM main.sale_return_items AS returned JOIN main.sale_items AS item ON item.id=returned.sale_item_id ORDER BY returned.rowid",
            )
            .map_err(|_| MigrationError::ApplyFailed)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            })
            .map_err(|_| MigrationError::ApplyFailed)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| MigrationError::ApplyFailed)?;
        rows
    };
    if i64::try_from(returns.len()).map_err(|_| MigrationError::ApplyFailed)? != return_count {
        return Err(MigrationError::ApplyFailed);
    }

    let mut returned_by_item = HashMap::new();
    for (return_id, sale_item_id, sale_id, quantity, sold_quantity, total_cents) in returns {
        let return_id = migration_id(return_id)?;
        let sale_item_id = migration_id(sale_item_id)?;
        let sale_id = migration_id(sale_id)?;
        let prior_quantity = *returned_by_item.get(&sale_item_id).unwrap_or(&0);
        let item = SaleItem {
            id: sale_item_id,
            product_id: None,
            description: String::new(),
            quantity: sold_quantity,
            unit_price_cents: 0,
            discount_cents: 0,
            total_cents,
        };
        let return_amount_cents = item
            .cumulative_return_amount_cents(prior_quantity, quantity)
            .map_err(|_| MigrationError::ApplyFailed)?;
        let cumulative_quantity = prior_quantity
            .checked_add(quantity)
            .filter(|&value| value <= sold_quantity)
            .ok_or(MigrationError::ApplyFailed)?;
        returned_by_item.insert(sale_item_id, cumulative_quantity);

        let due_cents = due_by_sale.entry(sale_id).or_insert(0);
        let debt_reduction_cents = return_amount_cents.min(*due_cents);
        *due_cents -= debt_reduction_cents;
        if transaction
            .execute(
                "UPDATE main.sale_return_items SET return_amount_cents=?1,debt_reduction_cents=?2 WHERE id=?3",
                params![return_amount_cents, debt_reduction_cents, return_id.as_slice()],
            )
            .map_err(|_| MigrationError::ApplyFailed)?
            != 1
        {
            return Err(MigrationError::ApplyFailed);
        }
    }
    Ok(())
}

fn migration_id(bytes: Vec<u8>) -> Result<[u8; 16], MigrationError> {
    bytes.try_into().map_err(|_| MigrationError::ApplyFailed)
}

pub struct AuditEvent {
    pub event_id: [u8; 16],
    pub occurred_at_ms: i64,
    pub operation: &'static str,
    pub entity: &'static str,
    pub entity_id: [u8; 16],
}

#[derive(PartialEq, Eq)]
pub enum AuditEventError {
    InvalidEvent,
    InsertFailed,
}

impl fmt::Debug for AuditEventError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidEvent => "InvalidEvent",
            Self::InsertFailed => "InsertFailed",
        })
    }
}

impl fmt::Display for AuditEventError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidEvent => "audit event is invalid",
            Self::InsertFailed => "audit event could not be inserted",
        })
    }
}

impl Error for AuditEventError {}

pub fn append_audit_event(
    transaction: &Transaction<'_>,
    event: &AuditEvent,
) -> Result<(), AuditEventError> {
    if event.event_id == [0; 16]
        || event.entity_id == [0; 16]
        || event.occurred_at_ms < 0
        || !valid_audit_tag(event.operation)
        || !valid_audit_tag(event.entity)
    {
        return Err(AuditEventError::InvalidEvent);
    }

    match transaction.execute(
            "INSERT INTO main.audit_event (event_id, occurred_at_ms, operation, entity, entity_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                event.event_id.as_slice(),
                event.occurred_at_ms,
                event.operation,
                event.entity,
                event.entity_id.as_slice(),
            ],
        ) {
        Ok(1) => Ok(()),
        Ok(_) | Err(_) => Err(AuditEventError::InsertFailed),
    }
}

fn valid_audit_tag(tag: &str) -> bool {
    let bytes = tag.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 64
        && bytes[0].is_ascii_lowercase()
        && bytes[1..]
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
}
