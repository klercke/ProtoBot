use chrono::{Datelike, NaiveDate, Utc, Weekday};
use chrono_tz::Tz;

/// The primary timezone that ProtoBot is used in
pub const BOT_TZ: Tz = chrono_tz::America::New_York;

/// All currently supported holidays
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Holiday {
    NewYearsEve,
    NewYearsDay,
    ValentinesDay,
    MothersDay,
    FathersDay,
    ChristmasEve,
    ChristmasDay,
}

impl Holiday {
    /// True for both Christmas Eve and Christmas Day
    pub fn is_christmas(self) -> bool {
        matches!(self, Holiday::ChristmasEve | Holiday::ChristmasDay)
    }
}

/// Returns the date of the `n`th `weekday` in a month (e.g. 2nd Sunday of May)
fn nth_weekday(year: i32, month: u32, weekday: Weekday, n: u32) -> NaiveDate {
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let offset = (7 + weekday.num_days_from_sunday() as i64
        - first.weekday().num_days_from_sunday() as i64)
        % 7;
    let day = 1 + offset as u32 + (n - 1) * 7;
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

/// Returns the tracked holiday for an arbitrary date, if any
pub fn holiday_for(date: NaiveDate) -> Option<Holiday> {
    let (y, m, d) = (date.year(), date.month(), date.day());

    match (m, d) {
        (1, 1) => return Some(Holiday::NewYearsDay),
        (12, 31) => return Some(Holiday::NewYearsEve),
        (2, 14) => return Some(Holiday::ValentinesDay),
        (12, 24) => return Some(Holiday::ChristmasEve),
        (12, 25) => return Some(Holiday::ChristmasDay),
        _ => {}
    }
    if date == nth_weekday(y, 5, Weekday::Sun, 2) {
        return Some(Holiday::MothersDay);
    }
    if date == nth_weekday(y, 6, Weekday::Sun, 3) {
        return Some(Holiday::FathersDay);
    }
    None
}

/// Today's date in BOT_TZ
pub fn today() -> NaiveDate {
    Utc::now().with_timezone(&BOT_TZ).date_naive()
}

/// The holiday in effect right now, if any
/// Pure date math, so it's cheap to call on every message
pub fn current_holiday() -> Option<Holiday> {
    holiday_for(today())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn detection() {
        assert_eq!(holiday_for(d(2026, 6, 21)), Some(Holiday::FathersDay));
        assert_eq!(holiday_for(d(2026, 5, 10)), Some(Holiday::MothersDay));
        assert_eq!(holiday_for(d(2026, 12, 24)), Some(Holiday::ChristmasEve));
        assert_eq!(holiday_for(d(2026, 12, 25)), Some(Holiday::ChristmasDay));
        assert_eq!(holiday_for(d(2026, 7, 4)), None);
    }

    #[test]
    fn christmas_grouping() {
        assert!(holiday_for(d(2026, 12, 25)).unwrap().is_christmas());
    }
}
