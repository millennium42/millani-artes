use super::{brasilia_day_window_ms, sum_positive_movement_amounts_cents};

#[test]
fn fixed_brasilia_day_uses_inclusive_start_and_exclusive_end() {
    assert_eq!(
        brasilia_day_window_ms(10_799_999).unwrap(),
        (-75_600_000, 10_800_000)
    );
    assert_eq!(
        brasilia_day_window_ms(10_800_000).unwrap(),
        (10_800_000, 97_200_000)
    );
    assert_eq!(
        brasilia_day_window_ms(97_199_999).unwrap(),
        (10_800_000, 97_200_000)
    );
    assert_eq!(
        brasilia_day_window_ms(97_200_000).unwrap(),
        (97_200_000, 183_600_000)
    );
}

#[test]
fn fixed_brasilia_window_crosses_month_and_year_boundaries() {
    const DAY_MS: i64 = 86_400_000;
    const FEBRUARY_2024_START_MS: i64 = 1_706_756_400_000;
    const JANUARY_2025_START_MS: i64 = 1_735_700_400_000;

    for start_ms in [FEBRUARY_2024_START_MS, JANUARY_2025_START_MS] {
        assert_eq!(
            brasilia_day_window_ms(start_ms - 1).unwrap(),
            (start_ms - DAY_MS, start_ms)
        );
        assert_eq!(
            brasilia_day_window_ms(start_ms).unwrap(),
            (start_ms, start_ms + DAY_MS)
        );
    }
}

#[test]
fn fixed_brasilia_window_rejects_negative_or_unrepresentable_instants() {
    assert!(brasilia_day_window_ms(-1).is_err());
    assert!(brasilia_day_window_ms(i64::MAX).is_err());
}

#[test]
fn movement_sum_accepts_empty_and_positive_cents_without_overflow() {
    assert_eq!(sum_positive_movement_amounts_cents(&[]).unwrap(), 0);
    assert_eq!(
        sum_positive_movement_amounts_cents(&[125, 300]).unwrap(),
        425
    );
    assert_eq!(
        sum_positive_movement_amounts_cents(&[i64::MAX]).unwrap(),
        i64::MAX
    );
}

#[test]
fn movement_sum_rejects_nonpositive_values_and_i64_overflow() {
    for amounts in [&[0][..], &[-1][..], &[i64::MAX, 1][..]] {
        assert!(sum_positive_movement_amounts_cents(amounts).is_err());
    }
}
