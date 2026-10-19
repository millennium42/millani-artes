use crate::application::pay_recurrence::RecurrencePaymentStore;
use crate::database::{append_audit_event, execute_write_transaction, AuditEvent};
use crate::domain::{Movement, MovementKind, ObligationState, RecurrenceObligation};
use crate::infrastructure::recurrences::{
    context_key, AdapterSqliteRecurrenceStore, RecurrenceStoreError,
};
use rusqlite::params;

impl RecurrencePaymentStore for AdapterSqliteRecurrenceStore {
    type Error = RecurrenceStoreError;

    fn persist_recurrence_payment_with_audit(
        &mut self,
        obligation: &RecurrenceObligation,
        payment: &Movement,
        event_id: [u8; 16],
        occurred_at_ms: i64,
    ) -> Result<(), Self::Error> {
        obligation
            .state
            .validate()
            .map_err(|_| RecurrenceStoreError)?;

        let (amount, movement_id) = match &obligation.state {
            ObligationState::Paid {
                amount_cents,
                movement_id,
            } => (*amount_cents, *movement_id),
            _ => return Err(RecurrenceStoreError),
        };
        if movement_id != payment.id
            || context_key(&obligation.context) != context_key(&payment.context)
        {
            return Err(RecurrenceStoreError);
        }
        let account_id = match &payment.kind {
            MovementKind::Exit { account_id } => *account_id,
            _ => return Err(RecurrenceStoreError),
        };
        ObligationState::Payable {
            amount_cents: amount,
        }
        .mark_paid(payment)
        .map_err(|_| RecurrenceStoreError)?;

        let connection = self.connection.take().ok_or(RecurrenceStoreError)?;

        let result = execute_write_transaction(connection, |tx| {
            let inserted = tx
                .execute(
                    "INSERT INTO main.movements (id, kind, amount_cents, context, account_id) VALUES (?1, 'exit', ?2, ?3, ?4)",
                    params![
                        payment.id.as_slice(),
                        payment.amount_cents,
                        context_key(&payment.context),
                        account_id.as_slice()
                    ],
                )
                .map_err(|_| ())?;

            if inserted != 1 {
                return Err(());
            }

            let updated = tx
                .execute(
                    "UPDATE main.recurrence_obligations SET state='paid', movement_id=?1 WHERE id=?2 AND recurrence_id=?3 AND period_year=?4 AND period_month=?5 AND due_at_ms=?6 AND context=?7 AND state IN ('payable','overdue') AND amount_cents=?8 AND movement_id IS NULL",
                    params![
                        payment.id.as_slice(),
                        obligation.id.as_slice(),
                        obligation.recurrence_id.as_slice(),
                        obligation.period_year,
                        obligation.period_month,
                        obligation.due_at_ms,
                        context_key(&obligation.context),
                        amount
                    ],
                )
                .map_err(|_| ())?;

            if updated != 1 {
                return Err(());
            }

            append_audit_event(
                tx,
                &AuditEvent {
                    event_id,
                    occurred_at_ms,
                    operation: "recurrence_pay",
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
