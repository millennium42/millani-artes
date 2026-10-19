use crate::application::generate_fixed_obligation::FixedObligationStore;
use crate::application::generate_variable_obligation::VariableObligationStore;
use crate::database::{append_audit_event, execute_write_transaction, AuditEvent};
use crate::domain::{
    Context, FixedRecurrence, ObligationState, RecurrenceObligation, VariableRecurrence,
};
use rusqlite::Connection;
use std::fmt;

pub struct RecurrenceStoreError;

impl fmt::Debug for RecurrenceStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RecurrenceStoreError")
    }
}

impl fmt::Display for RecurrenceStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("recurrence could not be stored")
    }
}

impl std::error::Error for RecurrenceStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub struct AdapterSqliteRecurrenceStore {
    connection: Option<Connection>,
}

impl AdapterSqliteRecurrenceStore {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Some(connection),
        }
    }
}

struct Configuration<'a> {
    id: [u8; 16],
    name: &'a str,
    kind: &'static str,
    amount: Option<i64>,
    context: &'static str,
    due_day: u8,
}

fn context_key(context: &Context) -> &'static str {
    match context {
        Context::Home => "home",
        Context::Factory => "factory",
    }
}

fn validate_payload(
    expected: &RecurrenceObligation,
    actual: &RecurrenceObligation,
) -> Result<Option<i64>, RecurrenceStoreError> {
    if expected.recurrence_id != actual.recurrence_id {
        return Err(RecurrenceStoreError);
    }
    if expected.due_at_ms != actual.due_at_ms {
        return Err(RecurrenceStoreError);
    }
    if context_key(&expected.context) != context_key(&actual.context) {
        return Err(RecurrenceStoreError);
    }

    match (&expected.state, &actual.state) {
        (ObligationState::AwaitingValue, ObligationState::AwaitingValue) => Ok(None),
        (
            ObligationState::Payable {
                amount_cents: expected_amount,
            },
            ObligationState::Payable {
                amount_cents: actual_amount,
            },
        ) if expected_amount == actual_amount => Ok(Some(*actual_amount)),
        _ => Err(RecurrenceStoreError),
    }
}

impl FixedObligationStore for AdapterSqliteRecurrenceStore {
    type Error = RecurrenceStoreError;

    fn persist_fixed_obligation_with_audit(
        &mut self,
        recurrence: &FixedRecurrence,
        obligation: &RecurrenceObligation,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error> {
        let expected = recurrence
            .generate_obligation(
                obligation.id,
                (obligation.period_year, obligation.period_month),
            )
            .map_err(|_| RecurrenceStoreError)?;

        let amount = validate_payload(&expected, obligation)?;

        let config = Configuration {
            id: recurrence.id,
            name: &recurrence.name,
            kind: "fixed",
            amount: Some(recurrence.amount_cents),
            context: context_key(&recurrence.context),
            due_day: recurrence.due_day,
        };

        let operation = "recurrence_fixed_generate";

        self.persist(
            config,
            obligation,
            amount,
            event_id,
            occurred_at_ms,
            operation,
        )
    }
}

impl VariableObligationStore for AdapterSqliteRecurrenceStore {
    type Error = RecurrenceStoreError;

    fn persist_variable_obligation_with_audit(
        &mut self,
        recurrence: &VariableRecurrence,
        obligation: &RecurrenceObligation,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error> {
        let expected = recurrence
            .generate_obligation(
                obligation.id,
                (obligation.period_year, obligation.period_month),
            )
            .map_err(|_| RecurrenceStoreError)?;

        let amount = validate_payload(&expected, obligation)?;

        let config = Configuration {
            id: recurrence.id,
            name: &recurrence.name,
            kind: "variable",
            amount: None,
            context: context_key(&recurrence.context),
            due_day: recurrence.due_day,
        };

        let operation = "recurrence_variable_generate";

        self.persist(
            config,
            obligation,
            amount,
            event_id,
            occurred_at_ms,
            operation,
        )
    }
}

impl AdapterSqliteRecurrenceStore {
    fn persist(
        &mut self,
        config: Configuration,
        obligation: &RecurrenceObligation,
        amount: Option<i64>,
        event_id: [u8; 16],
        occurred_at_ms: i64,
        operation: &'static str,
    ) -> Result<(), RecurrenceStoreError> {
        let connection = self.connection.take().ok_or(RecurrenceStoreError)?;
        let result = execute_write_transaction(connection, |transaction| {
            transaction
                .execute(
                    "INSERT INTO main.recurrences (id, name, kind, amount_cents, context, due_day) VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT(id) DO NOTHING",
                    rusqlite::params![
                        config.id.as_slice(),
                        config.name,
                        config.kind,
                        config.amount,
                        config.context,
                        i64::from(config.due_day)
                    ],
                )
                .map_err(|_| ())?;

            let existing: (String, String, Option<i64>, String, i64) = transaction
                .query_row(
                    "SELECT name, kind, amount_cents, context, due_day FROM main.recurrences WHERE id = ?1",
                    rusqlite::params![config.id.as_slice()],
                    |row| {
                        let amount_cents: Option<i64> = row.get(2)?;
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            amount_cents,
                            row.get(3)?,
                            row.get(4)?,
                        ))
                    },
                )
                .map_err(|_| ())?;

            if existing.0 != config.name
                || existing.1 != config.kind
                || existing.2 != config.amount
                || existing.3 != config.context
                || existing.4 != i64::from(config.due_day)
            {
                return Err(());
            }

            let state = match amount {
                Some(_) => "payable",
                None => "awaiting_value",
            };

            let inserted = transaction
                .execute(
                    "INSERT INTO main.recurrence_obligations (id, recurrence_id, period_year, period_month, due_at_ms, state, amount_cents, movement_id, context) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, ?8)",
                    rusqlite::params![
                        obligation.id.as_slice(),
                        obligation.recurrence_id.as_slice(),
                        obligation.period_year,
                        obligation.period_month,
                        obligation.due_at_ms,
                        state,
                        amount,
                        config.context
                    ],
                )
                .map_err(|_| ())?;

            if inserted != 1 {
                return Err(());
            }

            append_audit_event(
                transaction,
                &AuditEvent {
                    event_id,
                    occurred_at_ms,
                    operation,
                    entity: "recurrence_obligation",
                    entity_id: obligation.id,
                },
            )
            .map_err(|_| ())?;

            Ok(())
        });

        match result {
            Ok((connection, ())) => {
                self.connection = Some(connection);
                Ok(())
            }
            Err(_) => Err(RecurrenceStoreError),
        }
    }
}

mod amount;
