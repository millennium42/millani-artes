use super::{brasilia_month_window_ms, sum_nonnegative_sale_amounts_cents};

const JANUARY_2024_START_MS: i64 = 1_704_078_000_000;
const FEBRUARY_2024_START_MS: i64 = 1_706_756_400_000;
const MARCH_2024_START_MS: i64 = 1_709_262_000_000;
const DECEMBER_2023_START_MS: i64 = 1_701_399_600_000;

#[test]
fn month_window_uses_brasilia_midnight_and_half_open_boundaries() {
    assert_eq!(
        brasilia_month_window_ms(JANUARY_2024_START_MS).unwrap(),
        (JANUARY_2024_START_MS, FEBRUARY_2024_START_MS)
    );
    assert_eq!(
        brasilia_month_window_ms(FEBRUARY_2024_START_MS - 1).unwrap(),
        (JANUARY_2024_START_MS, FEBRUARY_2024_START_MS)
    );
    assert_eq!(
        brasilia_month_window_ms(FEBRUARY_2024_START_MS).unwrap(),
        (FEBRUARY_2024_START_MS, MARCH_2024_START_MS)
    );
}

#[test]
fn month_window_crosses_year_and_handles_epoch_local_month() {
    assert_eq!(
        brasilia_month_window_ms(JANUARY_2024_START_MS - 1).unwrap(),
        (DECEMBER_2023_START_MS, JANUARY_2024_START_MS)
    );
    assert_eq!(
        brasilia_month_window_ms(0).unwrap(),
        (-2_667_600_000, 10_800_000)
    );
}

#[test]
fn invalid_or_unrepresentable_month_windows_fail() {
    assert!(brasilia_month_window_ms(-1).is_err());
    assert!(brasilia_month_window_ms(i64::MAX).is_err());
}

#[test]
fn sale_amount_sum_allows_zero_and_empty_but_rejects_negative_or_overflow() {
    assert_eq!(sum_nonnegative_sale_amounts_cents(&[]).unwrap(), 0);
    assert_eq!(
        sum_nonnegative_sale_amounts_cents(&[0, 19, 23]).unwrap(),
        42
    );
    assert!(sum_nonnegative_sale_amounts_cents(&[-1]).is_err());
    assert!(sum_nonnegative_sale_amounts_cents(&[i64::MAX, 1]).is_err());
}
