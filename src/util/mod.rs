use std::mem;

use tokio::time::{sleep, Duration};
use twilight_http::{error::ErrorType, request::channel::reaction::RequestReactionType};
use twilight_model::channel::Message;

use crate::{error::Error, BotResult, Context};

pub use self::{cow::CowUtils, ext::*, mods_fmt::ModsFormatter};

pub mod builder;
pub mod constants;
pub mod datetime;
pub mod matcher;
pub mod numbers;
pub mod osu;

mod cow;
mod ext;
mod mods_fmt;

macro_rules! get {
    ($slice:ident[$idx:expr]) => {
        unsafe { *$slice.get_unchecked($idx) }
    };
}

macro_rules! set {
    ($slice:ident[$idx:expr] = $val:expr) => {
        unsafe { *$slice.get_unchecked_mut($idx) = $val }
    };
}

/// "How many replace/delete/insert operations are necessary to morph one word into the other?"
///
/// Returns (distance, max word length) tuple
pub fn levenshtein_distance<'w>(mut word_a: &'w str, mut word_b: &'w str) -> (usize, usize) {
    let m = word_a.chars().count();
    let mut n = word_b.chars().count();

    if m > n {
        mem::swap(&mut word_a, &mut word_b);
        n = m;
    }

    // u16 is sufficient considering the max length
    // of discord messages is smaller than u16::MAX
    let mut costs: Vec<_> = (0..=n as u16).collect();

    // SAFETY for get! and set!:
    // chars(word_a) <= chars(word_b) = n < n + 1 = costs.len()

    for (a, i) in word_a.chars().zip(1..) {
        let mut last_val = i;

        for (b, j) in word_b.chars().zip(1..) {
            let new_val = if a == b {
                get!(costs[j - 1])
            } else {
                get!(costs[j - 1]).min(last_val).min(get!(costs[j])) + 1
            };

            set!(costs[j - 1] = last_val);
            last_val = new_val;
        }

        set!(costs[n] = last_val);
    }

    (get!(costs[n]) as usize, n)
}

pub async fn send_reaction(
    ctx: &Context,
    msg: &Message,
    emoji: &RequestReactionType<'_>,
) -> BotResult<()> {
    let channel = msg.channel_id;
    let msg = msg.id;

    // Initial attempt, return if it's not a 429
    match ctx.http.create_reaction(channel, msg, emoji).await {
        Ok(_) => return Ok(()),
        Err(e) if matches!(e.kind(), ErrorType::Response { status, .. } if *status == 429) => {}
        Err(e) => return Err(e.into()),
    }

    const TRIES: usize = 3;

    // 200ms - 1000ms - 2000ms
    let backoff = ExponentialBackoff::new(5).factor(40).max_delay(2000);

    for (duration, i) in backoff.take(TRIES).zip(1..) {
        debug!("Send reaction retry attempt #{i} | Backoff {duration:?}");
        sleep(duration).await;

        match ctx.http.create_reaction(channel, msg, emoji).await {
            Ok(_) => return Ok(()),
            Err(e) if matches!(e.kind(), ErrorType::Response { status, .. } if *status == 429) => {}
            Err(e) => return Err(e.into()),
        };
    }

    Err(Error::ReactionRatelimit(TRIES))
}

#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    current: Duration,
    base: u32,
    factor: u32,
    max_delay: Option<Duration>,
}

impl ExponentialBackoff {
    pub fn new(base: u32) -> Self {
        ExponentialBackoff {
            current: Duration::from_millis(base as u64),
            base,
            factor: 1,
            max_delay: None,
        }
    }

    pub fn factor(mut self, factor: u32) -> Self {
        self.factor = factor;

        self
    }

    pub fn max_delay(mut self, max_delay: u64) -> Self {
        self.max_delay.replace(Duration::from_millis(max_delay));

        self
    }
}

impl Iterator for ExponentialBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        let duration = self.current * self.factor;

        if let Some(max_delay) = self.max_delay.filter(|&max_delay| duration > max_delay) {
            return Some(max_delay);
        }

        self.current *= self.base;

        Some(duration)
    }
}
