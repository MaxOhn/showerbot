use std::fmt;

use chrono::{DateTime, Datelike, Utc};

pub struct SecToMinSecFormatter {
    secs: u32,
}

impl fmt::Display for SecToMinSecFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{:02}", self.secs / 60, self.secs % 60)
    }
}

pub struct HowLongAgoFormatterText<'a>(&'a DateTime<Utc>);

impl<'a> fmt::Display for HowLongAgoFormatterText<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let now = Utc::now();
        let diff_sec = now.timestamp() - self.0.timestamp();
        debug_assert!(diff_sec >= 0);

        let one_day = 24 * 3600;
        let one_week = 7 * one_day;

        let (amount, unit) = {
            if diff_sec < 60 {
                (diff_sec, "second")
            } else if diff_sec < 3600 {
                (diff_sec / 60, "minute")
            } else if diff_sec < one_day {
                (diff_sec / 3600, "hour")
            } else if diff_sec < one_week {
                (diff_sec / one_day, "day")
            } else if diff_sec < 4 * one_week {
                (diff_sec / one_week, "week")
            } else {
                let diff_month = (12 * (now.year() - self.0.year()) as u32 + now.month()
                    - self.0.month()) as i64;

                if diff_month < 1 {
                    (diff_sec / one_week, "week")
                } else if diff_month < 12 {
                    (diff_month, "month")
                } else {
                    let years = diff_month / 12 + (diff_month % 12 > 9) as i64;

                    (years, "year")
                }
            }
        };

        write!(
            f,
            "{amount} {unit}{plural} ago",
            plural = if amount == 1 { "" } else { "s" }
        )
    }
}

/// Instead of writing the whole string like `how_long_ago_text`,
/// this just writes discord's syntax for dynamic timestamps and lets
/// discord handle the rest.
///
/// Note: Doesn't work in embed footers
pub fn how_long_ago_dynamic(date: &DateTime<Utc>) -> HowLongAgoFormatterDynamic {
    HowLongAgoFormatterDynamic(date.timestamp())
}

#[derive(Copy, Clone)]
pub struct HowLongAgoFormatterDynamic(i64);

impl fmt::Display for HowLongAgoFormatterDynamic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // https://discord.com/developers/docs/reference#message-formatting-timestamp-styles
        write!(f, "<t:{}:R>", self.0)
    }
}
