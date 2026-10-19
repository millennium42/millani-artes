use crate::application::mark_obligation_overdue::{
    audit_millis, ObservedClock, OverdueObligationStore,
};
use crate::database::{append_audit_event, execute_write_transaction, AuditEvent};
use crate::domain::{ObligationState, RecurrenceObligation};
use crate::infrastructure::recurrences::{
    context_key, AdapterSqliteRecurrenceStore, RecurrenceStoreError,
};
use rusqlite::params;
use std::time::SystemTime;

impl OverdueObligationStore for AdapterSqliteRecurrenceStore {
    type Error = RecurrenceStoreError;

    fn persist_overdue_obligation_with_audit(
        &mut self,
        obligation: &RecurrenceObligation,
        event_id: [u8; 16],
        observed_at: SystemTime,
    ) -> Result<(), Self::Error> {
        obligation
            .state
            .validate()
            .map_err(|_| RecurrenceStoreError)?;
        let amount_cents = match &obligation.state {
            ObligationState::Overdue { amount_cents } => *amount_cents,
            _ => return Err(RecurrenceStoreError),
        };
        ObligationState::Payable { amount_cents }
            .mark_overdue_ms(obligation.due_at_ms, &ObservedClock(observed_at))
            .map_err(|_| RecurrenceStoreError)?;

        let occurred_at_ms = audit_millis(observed_at).map_err(|_| RecurrenceStoreError)?;

        let connection = self.connection.take().ok_or(RecurrenceStoreError)?;

        let result = execute_write_transaction(connection, |tx| {
            let changed = tx
                .execute(
                    "UPDATE main.recurrence_obligations
                     SET state = 'overdue'
                     WHERE id = ?1
                       AND recurrence_id = ?2
                       AND period_year = ?3
                       AND period_month = ?4
                       AND due_at_ms = ?5
                       AND context = ?6
                       AND state = 'payable'
                       AND amount_cents = ?7
                       AND movement_id IS NULL",
                    params![
                        obligation.id.as_slice(),
                        obligation.recurrence_id.as_slice(),
                        obligation.period_year,
                        obligation.period_month,
                        obligation.due_at_ms,
                        context_key(&obligation.context),
                        amount_cents
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
                    operation: "recurrence_overdue",
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
