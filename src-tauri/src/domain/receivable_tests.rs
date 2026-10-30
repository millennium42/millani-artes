use super::{Context, Receivable, ReceivableReceipt, SalePending};

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

#[test]
fn rcv004_calculates_remaining_due_from_principal_and_applied_total() {
    let receivable = Receivable {
        id: [0x31; 16],
        sale_id: [0x42; 16],
        pending: SalePending {
            amount_cents: 3000,
            customer_id: [0x52; 16],
            due_at_ms: 1_800_000_000_000,
        },
    };

    assert_eq!(receivable.remaining_due_cents(0), Ok(3000));
    assert_eq!(receivable.remaining_due_cents(1000), Ok(2000));
    assert_eq!(receivable.remaining_due_cents(3000), Ok(0));
}

#[test]
fn rcv004_accepts_i64_max_and_rejects_invalid_amounts() {
    let receivable = |amount_cents| Receivable {
        id: [0x31; 16],
        sale_id: [0x42; 16],
        pending: SalePending {
            amount_cents,
            customer_id: [0x52; 16],
            due_at_ms: 1_800_000_000_000,
        },
    };

    assert_eq!(receivable(i64::MAX).remaining_due_cents(0), Ok(i64::MAX));
    assert_eq!(receivable(i64::MAX).remaining_due_cents(i64::MAX), Ok(0));
    assert_eq!(
        receivable(i64::MIN).remaining_due_cents(0),
        Err(super::DomainError::InvalidInput)
    );
    assert_eq!(
        receivable(i64::MAX).remaining_due_cents(i64::MIN),
        Err(super::DomainError::InvalidInput)
    );
    assert_eq!(
        receivable(3000).remaining_due_cents(-1),
        Err(super::DomainError::InvalidInput)
    );
    assert_eq!(
        receivable(3000).remaining_due_cents(3001),
        Err(super::DomainError::InvalidInput)
    );
    assert_eq!(
        receivable(0).remaining_due_cents(0),
        Err(super::DomainError::InvalidInput)
    );
    assert_eq!(
        receivable(-1).remaining_due_cents(0),
        Err(super::DomainError::InvalidInput)
    );
}

#[test]
fn rcv005_preserves_receipt_account_amount_context_and_order() {
    let receipts = vec![
        ReceivableReceipt {
            account_id: [0x51; 16],
            amount_cents: 500,
            context: Context::Home,
        },
        ReceivableReceipt {
            account_id: [0x52; 16],
            amount_cents: 1000,
            context: Context::Factory,
        },
    ];

    assert_eq!(receipts[0].account_id, [0x51; 16]);
    assert_eq!(receipts[0].amount_cents, 500);
    assert!(matches!(&receipts[0].context, Context::Home));
    assert_eq!(receipts[1].account_id, [0x52; 16]);
    assert_eq!(receipts[1].amount_cents, 1000);
    assert!(matches!(&receipts[1].context, Context::Factory));
}
