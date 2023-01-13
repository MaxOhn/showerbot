use std::fmt::{Display, Formatter, Result as FmtResult};

use time::{
    format_description::{
        modifier::{Day, Hour, Minute, Month, OffsetHour, OffsetMinute, Second, Year},
        Component, FormatItem,
    },
    OffsetDateTime,
};

/// Instead of writing the whole string like `HowLongAgoText`,
/// this just writes discord's syntax for dynamic timestamps and lets
/// discord handle the rest.
///
/// Note: Doesn't work in embed footers
#[derive(Copy, Clone)]
pub struct HowLongAgoDynamic {
    secs: i64,
}

impl HowLongAgoDynamic {
    pub fn new(datetime: &OffsetDateTime) -> Self {
        Self {
            secs: datetime.unix_timestamp(),
        }
    }
}

impl Display for HowLongAgoDynamic {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // https://discord.com/developers/docs/reference#message-formatting-timestamp-styles
        write!(f, "<t:{}:R>", self.secs)
    }
}

pub const DATE_FORMAT: &[FormatItem<'_>] = &[
    FormatItem::Component(Component::Year(Year::default())),
    FormatItem::Literal(b"-"),
    FormatItem::Component(Component::Month(Month::default())),
    FormatItem::Literal(b"-"),
    FormatItem::Component(Component::Day(Day::default())),
];

pub const TIME_FORMAT: &[FormatItem<'_>] = &[
    FormatItem::Component(Component::Hour(<Hour>::default())),
    FormatItem::Literal(b":"),
    FormatItem::Component(Component::Minute(<Minute>::default())),
    FormatItem::Literal(b":"),
    FormatItem::Component(Component::Second(<Second>::default())),
];

pub const OFFSET_FORMAT: &[FormatItem<'_>] = &[
    FormatItem::Component(Component::OffsetHour(OffsetHour::default())),
    FormatItem::Literal(b":"),
    FormatItem::Component(Component::OffsetMinute(OffsetMinute::default())),
];

pub const DATETIME_FORMAT: &[FormatItem<'_>] = &[
    FormatItem::Compound(DATE_FORMAT),
    FormatItem::Literal(b"T"),
    FormatItem::Compound(TIME_FORMAT),
];
