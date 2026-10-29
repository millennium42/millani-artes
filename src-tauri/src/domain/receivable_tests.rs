use super::{Receivable, SalePending};

#[test]
fn rcv001_preserves_receivable_identity_and_initial_pending_terms() {
    let pending = SalePending {
        amount_cents: i64::MAX,
        customer_id: [0x52; 16],
        due_at_ms: i64::MIN,
    };
    let receivable = Receivable {
        id: [0x31; 16],
        sale_id: [0x42; 16],
        pending,
    };

    assert_eq!(receivable.id, [0x31; 16]);
    assert_eq!(receivable.sale_id, [0x42; 16]);
    assert_eq!(receivable.pending.amount_cents, i64::MAX);
    assert_eq!(receivable.pending.customer_id, [0x52; 16]);
    assert_eq!(receivable.pending.due_at_ms, i64::MIN);
}
