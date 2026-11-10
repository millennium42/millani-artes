use super::sum_open_recurrence_obligation_amounts_cents;

#[test]
fn dashboard_payables_empty_is_zero_and_open_values_are_summed() {
    assert_eq!(
        sum_open_recurrence_obligation_amounts_cents(&[]).unwrap(),
        0
    );
    assert_eq!(
        sum_open_recurrence_obligation_amounts_cents(&[1_250, 4_750, 300]).unwrap(),
        6_300
    );
}

#[test]
fn dashboard_payables_rejects_nonpositive_values_without_partial_total() {
    assert!(sum_open_recurrence_obligation_amounts_cents(&[1_250, 0, 300]).is_err());
    assert!(sum_open_recurrence_obligation_amounts_cents(&[1_250, -1, 300]).is_err());
}

#[test]
fn dashboard_payables_rejects_overflow_without_partial_total() {
    assert!(sum_open_recurrence_obligation_amounts_cents(&[i64::MAX, 1]).is_err());
}
