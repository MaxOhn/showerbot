use std::fmt::{Display, Formatter, Result as FmtResult};

use time::OffsetDateTime;

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
