//! Clock and business calendar. The Best POS runs on Egypt time (Africa/Cairo): UTC+2, with the
//! summer time (UTC+3) shown on printed times only. Business days and SQL day boundaries use the
//! fixed UTC+2 offset so every report agrees with itself.

use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, Utc};

/// Africa/Cairo standard offset from UTC in milliseconds (names kept from the base project).
pub const ALGIERS_OFFSET_MS: i64 = 7_200_000;

/// Current time as unix milliseconds UTC.
pub fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

/// Last given weekday (`Datelike::weekday().num_days_from_monday()`) of a month.
fn last_weekday(year: i32, month: u32, weekday_from_monday: u32) -> NaiveDate {
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap_or(NaiveDate::MIN);
    let mut d = next.pred_opt().unwrap_or(NaiveDate::MIN);
    while d.weekday().num_days_from_monday() != weekday_from_monday {
        d = d.pred_opt().unwrap_or(NaiveDate::MIN);
    }
    d
}

/// Egypt's summer time (since 2023): from the last Friday of April 00:00 to the last Thursday of
/// October 24:00, local time. Returns the offset to show at `ms`.
fn cairo_display_offset_ms(ms: i64) -> i64 {
    let year = match DateTime::<Utc>::from_timestamp_millis(ms.saturating_add(ALGIERS_OFFSET_MS)) {
        Some(d) => d.year(),
        None => return ALGIERS_OFFSET_MS,
    };
    let start = last_weekday(year, 4, 4)
        .and_time(chrono::NaiveTime::MIN)
        .and_utc()
        .timestamp_millis()
        - ALGIERS_OFFSET_MS;
    let end = last_weekday(year, 10, 3)
        .and_time(chrono::NaiveTime::MIN)
        .and_utc()
        .timestamp_millis()
        + 86_400_000
        - ALGIERS_OFFSET_MS
        - 3_600_000;
    if ms >= start && ms < end {
        ALGIERS_OFFSET_MS + 3_600_000
    } else {
        ALGIERS_OFFSET_MS
    }
}

/// Wall-clock time in Egypt for a UTC timestamp, for display (summer time included).
/// Out-of-range timestamps (±262 000 years) collapse to `NaiveDateTime::MIN`.
pub fn algiers_datetime(ms: i64) -> NaiveDateTime {
    DateTime::<Utc>::from_timestamp_millis(ms.saturating_add(cairo_display_offset_ms(ms)))
        .map(|d| d.naive_utc())
        .unwrap_or(NaiveDateTime::MIN)
}

/// The same with the fixed standard offset: what business days and SQL day boundaries use.
fn fixed_datetime(ms: i64) -> NaiveDateTime {
    DateTime::<Utc>::from_timestamp_millis(ms.saturating_add(ALGIERS_OFFSET_MS))
        .map(|d| d.naive_utc())
        .unwrap_or(NaiveDateTime::MIN)
}

/// The Algiers day `[00:00, 24:00)` as Unix ms UTC (the appointment book's day).
pub fn algiers_day_bounds(day: NaiveDate) -> (i64, i64) {
    let start = day
        .and_time(chrono::NaiveTime::MIN)
        .and_utc()
        .timestamp_millis()
        - ALGIERS_OFFSET_MS;
    (start, start + 86_400_000)
}

/// Business date of a timestamp in Algiers.
pub fn business_date(ms: i64) -> NaiveDate {
    fixed_datetime(ms).date()
}

/// The business day of `ms` when the day turns at `cutoff_hours` o'clock (CFG-008, at most 6):
/// a sale at 01:30 with a 03:00 cut-off belongs to the day before — except on 1 January, where
/// the calendar day stays (numbering and the fiscal year never go back a year).
pub fn business_date_with(ms: i64, cutoff_hours: u8) -> NaiveDate {
    let local = fixed_datetime(ms);
    let shifted = (local - chrono::Duration::hours(i64::from(cutoff_hours.min(6)))).date();
    if shifted.year() < local.date().year() {
        local.date()
    } else {
        shifted
    }
}

/// Fiscal year = calendar year of the business date (spec §5.2).
pub fn fiscal_year(ms: i64) -> i32 {
    business_date(ms).year()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn business_date_is_algiers_local() {
        // 2026-12-31T23:30:00Z is 00:30 on 1 January 2027 in Algiers.
        assert_eq!(
            business_date(1_798_759_800_000),
            NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()
        );
        assert_eq!(fiscal_year(1_798_759_800_000), 2027);
        // 2026-06-15T21:59:59.999Z is 23:59:59.999 local (UTC+2), still the 15th.
        assert_eq!(
            business_date(1_781_560_799_999),
            NaiveDate::from_ymd_opt(2026, 6, 15).unwrap()
        );
        // 2026-06-15T22:00:00Z is midnight local.
        assert_eq!(
            business_date(1_781_560_800_000),
            NaiveDate::from_ymd_opt(2026, 6, 16).unwrap()
        );
    }

    #[test]
    fn an_algiers_day_starts_at_local_midnight() {
        let (start, end) = algiers_day_bounds(NaiveDate::from_ymd_opt(2026, 6, 16).unwrap());
        // 2026-06-15T22:00:00Z is midnight local (see `business_date_is_algiers_local`).
        assert_eq!((start, end), (1_781_560_800_000, 1_781_647_200_000));
        assert_eq!(
            business_date(start),
            NaiveDate::from_ymd_opt(2026, 6, 16).unwrap()
        );
        assert_eq!(
            business_date(end - 1),
            NaiveDate::from_ymd_opt(2026, 6, 16).unwrap()
        );
        assert_eq!(
            business_date(end),
            NaiveDate::from_ymd_opt(2026, 6, 17).unwrap()
        );
    }

    #[test]
    fn the_business_day_turns_at_the_cutoff_hour_but_never_crosses_a_year() {
        use chrono::TimeZone;
        let at = |y, m, d, h, min| {
            chrono::FixedOffset::east_opt(7_200)
                .unwrap()
                .with_ymd_and_hms(y, m, d, h, min, 0)
                .unwrap()
                .timestamp_millis()
        };
        let day = |ms, cut| business_date_with(ms, cut).format("%Y-%m-%d").to_string();
        assert_eq!(
            day(at(2026, 9, 29, 1, 30), 3),
            "2026-09-28",
            "a café open past midnight"
        );
        assert_eq!(day(at(2026, 9, 29, 3, 0), 3), "2026-09-29");
        assert_eq!(day(at(2026, 9, 29, 1, 30), 0), "2026-09-29");
        assert_eq!(
            day(at(2027, 1, 1, 1, 0), 3),
            "2027-01-01",
            "the fiscal year never goes back"
        );
        assert_eq!(
            day(at(2026, 9, 29, 6, 30), 9),
            "2026-09-29",
            "at most 6 o'clock"
        );
    }

    #[test]
    fn cairo_datetime_adds_summer_hours() {
        let local = algiers_datetime(1_790_596_800_000); // 2026-09-28T12:00:00Z: Egyptian summer time, UTC+3
        assert_eq!(
            local.format("%Y-%m-%d %H:%M").to_string(),
            "2026-09-28 15:00"
        );
    }

    #[test]
    fn now_is_after_2026() {
        assert!(now_ms() > 1_767_225_600_000);
    }

    #[test]
    fn offset_is_two_hours_in_winter_and_three_in_summer() {
        assert_eq!(ALGIERS_OFFSET_MS, 2 * 60 * 60 * 1_000);
        // 2026-01-15T12:00:00Z (winter) and 2026-07-15T12:00:00Z (Egyptian summer time).
        let winter = algiers_datetime(1_768_478_400_000);
        let summer = algiers_datetime(1_784_116_800_000);
        assert_eq!(
            winter.format("%Y-%m-%d %H:%M").to_string(),
            "2026-01-15 14:00"
        );
        assert_eq!(
            summer.format("%Y-%m-%d %H:%M").to_string(),
            "2026-07-15 15:00"
        );
    }

    #[test]
    fn fiscal_year_ends_at_local_midnight() {
        // 2026-12-31T21:59:59.999Z is 23:59:59.999 local on 31 December 2026.
        assert_eq!(fiscal_year(1_798_754_399_999), 2026);
        assert_eq!(fiscal_year(1_798_754_400_000), 2027);
        // Before the epoch: 1969-12-31T21:59:59.999Z is still 31 December locally.
        assert_eq!(
            business_date(-7_200_001),
            NaiveDate::from_ymd_opt(1969, 12, 31).unwrap()
        );
        assert_eq!(
            business_date(-7_200_000),
            NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
        );
    }

    #[test]
    fn out_of_range_timestamps_collapse_without_panicking() {
        assert_eq!(algiers_datetime(i64::MAX), NaiveDateTime::MIN);
        assert_eq!(algiers_datetime(i64::MIN), NaiveDateTime::MIN);
        assert_eq!(business_date(i64::MAX), NaiveDateTime::MIN.date());
        assert_eq!(fiscal_year(i64::MIN), NaiveDateTime::MIN.year());
    }
}
