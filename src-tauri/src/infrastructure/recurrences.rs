use crate::application::generate_fixed_obligation::FixedObligationStore;
use crate::database::{append_audit_event, execute_write_transaction, AuditEvent};
use crate::domain::{Context, FixedRecurrence, ObligationState, RecurrenceObligation};
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

        let context_key = match &obligation.context {
            Context::Home => "home",
            Context::Factory => "factory",
        };

        if expected.recurrence_id != obligation.recurrence_id {
            return Err(RecurrenceStoreError);
        }
        if expected.due_at_ms != obligation.due_at_ms {
            return Err(RecurrenceStoreError);
        }
        if context_key
            != match &expected.context {
                Context::Home => "home",
                Context::Factory => "factory",
            }
        {
            return Err(RecurrenceStoreError);
        }

        let amount_cents = match &obligation.state {
            ObligationState::Payable { amount_cents }
                if *amount_cents == recurrence.amount_cents =>
            {
                *amount_cents
            }
            _ => return Err(RecurrenceStoreError),
        };

        let connection = self.connection.take().ok_or(RecurrenceStoreError)?;
        let result = execute_write_transaction(connection, |transaction| {
            transaction
                .execute(
                    "INSERT INTO main.recurrences (id, name, kind, amount_cents, context, due_day) VALUES (?1, ?2, 'fixed', ?3, ?4, ?5) ON CONFLICT(id) DO NOTHING",
                    rusqlite::params![
                        recurrence.id.as_slice(),
                        &recurrence.name,
                        recurrence.amount_cents,
                        match &recurrence.context {
                            Context::Home => "home",
                            Context::Factory => "factory",
                        },
                        recurrence.due_day
                    ],
                )
                .map_err(|_| ())?;

            let existing: (String, String, Option<i64>, String, i64) = transaction
                .query_row(
                    "SELECT name, kind, amount_cents, context, due_day FROM main.recurrences WHERE id = ?1",
                    rusqlite::params![recurrence.id.as_slice()],
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

            if existing.0 != recurrence.name
                || existing.1 != "fixed"
                || existing.2 != Some(recurrence.amount_cents)
                || existing.3
                    != match &recurrence.context {
                        Context::Home => "home",
                        Context::Factory => "factory",
                    }
                || existing.4 != i64::from(recurrence.due_day)
            {
                return Err(());
            }

            let inserted = transaction
                .execute(
                    "INSERT INTO main.recurrence_obligations (id, recurrence_id, period_year, period_month, due_at_ms, state, amount_cents, movement_id, context) VALUES (?1, ?2, ?3, ?4, ?5, 'payable', ?6, NULL, ?7)",
                    rusqlite::params![
                        obligation.id.as_slice(),
                        obligation.recurrence_id.as_slice(),
                        obligation.period_year,
                        obligation.period_month,
                        obligation.due_at_ms,
                        amount_cents,
                        match &obligation.context {
                            Context::Home => "home",
                            Context::Factory => "factory",
                        }
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
                    operation: "recurrence_fixed_generate",
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
