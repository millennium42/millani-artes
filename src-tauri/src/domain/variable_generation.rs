use super::{recurrence_due_at_ms, Context, RecurrenceObligation, Result, VariableRecurrence};

impl VariableRecurrence {
    pub fn generate_obligation(
        &self,
        id: [u8; 16],
        period: (u16, u8),
    ) -> Result<RecurrenceObligation> {
        let state = self.initial_obligation_state()?;
        let due_at_ms = recurrence_due_at_ms(period.0, period.1, self.due_day)?;
        Ok(RecurrenceObligation {
            id,
            recurrence_id: self.id,
            period_year: period.0,
            period_month: period.1,
            due_at_ms,
            state,
            context: match &self.context {
                Context::Home => Context::Home,
                Context::Factory => Context::Factory,
            },
        })
    }
}
