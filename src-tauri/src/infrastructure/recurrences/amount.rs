use crate::application::supply_obligation_amount::ObligationAmountStore;
use crate::database::{append_audit_event, execute_write_transaction, AuditEvent};
use crate::domain::{ObligationState, RecurrenceObligation};
use crate::infrastructure::recurrences::{
    context_key, AdapterSqliteRecurrenceStore, RecurrenceStoreError,
};
use rusqlite::params;

impl ObligationAmountStore for AdapterSqliteRecurrenceStore {
    type Error = RecurrenceStoreError;

    fn persist_obligation_amount_with_audit(
        &mut self,
        obligation: &RecurrenceObligation,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error> {
        obligation
            .state
            .validate()
            .map_err(|_| RecurrenceStoreError)?;

        let amount_cents = match &obligation.state {
            ObligationState::Payable { amount_cents } => *amount_cents,
            _ => return Err(RecurrenceStoreError),
        };

        let connection = self.connection.take().ok_or(RecurrenceStoreError)?;

        let result = execute_write_transaction(connection, |tx| {
            let changed = tx
                .execute(
                    "UPDATE main.recurrence_obligations
                 SET state = 'payable', amount_cents = ?1
                 WHERE id = ?2
                   AND recurrence_id = ?3
                   AND period_year = ?4
                   AND period_month = ?5
                   AND due_at_ms = ?6
                   AND context = ?7
                   AND state = 'awaiting_value'
                   AND amount_cents IS NULL
                   AND movement_id IS NULL
                   AND EXISTS(
                       SELECT 1 FROM main.recurrences r
                       WHERE r.id = recurrence_obligations.recurrence_id
                         AND r.kind = 'variable'
                         AND r.amount_cents IS NULL
                   )",
                    params![
                        amount_cents,
                        obligation.id.as_slice(),
                        obligation.recurrence_id.as_slice(),
                        obligation.period_year,
                        obligation.period_month,
                        obligation.due_at_ms,
                        context_key(&obligation.context)
                    ],
                )
                .map_err(|_| ())?;

            if changed != 1 {
                return Err(());
            }

            append_audit_event(
                tx,
                &AuditEvent {
                    event_id,
                    occurred_at_ms,
                    operation: "recurrence_amount_supply",
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
