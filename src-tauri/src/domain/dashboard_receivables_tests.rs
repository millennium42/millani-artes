use super::sum_nonnegative_receivable_due_cents;
use crate::domain::DomainError;

#[test]
fn dashboard_receivables_sum_empty_zero_and_positive_due_values() {
    assert_eq!(sum_nonnegative_receivable_due_cents(&[]), Ok(0));
    assert_eq!(
        sum_nonnegative_receivable_due_cents(&[0, 125, 300]),
        Ok(425)
    );
}

#[test]
fn dashboard_receivables_reject_negative_and_overflow_without_partial_total() {
    assert_eq!(
        sum_nonnegative_receivable_due_cents(&[125, -1, 300]),
        Err(DomainError::InvalidInput)
    );
    assert_eq!(
        sum_nonnegative_receivable_due_cents(&[i64::MAX, 1]),
        Err(DomainError::InvalidInput)
    );
}
