use std::{
    cmp::PartialOrd,
    ops::{AddAssign, Div},
};

use rosu_v2::prelude::OsuError;
use twilight_interactions::command::{CommandOption, CreateOption};

use crate::{util::osu::ModSelection, Error};

pub use self::leaderboard::*;

mod leaderboard;

pub trait HasMods {
    fn mods(&self) -> ModsResult;
}

pub enum ModsResult {
    Mods(ModSelection),
    None,
    Invalid,
}

enum ErrorType {
    Bot(Error),
    Osu(OsuError),
}

impl From<Error> for ErrorType {
    fn from(e: Error) -> Self {
        Self::Bot(e)
    }
}

impl From<OsuError> for ErrorType {
    fn from(e: OsuError) -> Self {
        Self::Osu(e)
    }
}

pub trait Number: AddAssign + Copy + Div<Output = Self> + PartialOrd {
    fn zero() -> Self;
    fn max() -> Self;
    fn min() -> Self;
    fn inc(&mut self);
}

#[rustfmt::skip]
impl Number for f32 {
    fn zero() -> Self { 0.0 }
    fn max() -> Self { f32::MAX }
    fn min() -> Self { f32::MIN }
    fn inc(&mut self) { *self += 1.0 }
}

#[rustfmt::skip]
impl Number for u32 {
    fn zero() -> Self { 0 }
    fn max() -> Self { u32::MAX }
    fn min() -> Self { u32::MIN }
    fn inc(&mut self) { *self += 1 }
}

pub struct MinMaxAvg<N> {
    min: N,
    max: N,
    sum: N,
    len: N,
}

impl From<MinMaxAvg<f32>> for MinMaxAvg<u32> {
    fn from(other: MinMaxAvg<f32>) -> Self {
        Self {
            min: other.min as u32,
            max: other.max as u32,
            sum: other.sum as u32,
            len: other.len as u32,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, CommandOption, CreateOption)]
pub enum ScoreOrder {
    #[option(name = "Accuracy", value = "acc")]
    Acc,
    #[option(name = "BPM", value = "bpm")]
    Bpm,
    #[option(name = "Combo", value = "combo")]
    Combo,
    #[option(name = "Date", value = "date")]
    Date,
    #[option(name = "Length", value = "len")]
    Length,
    #[option(name = "Misses", value = "misses")]
    Misses,
    #[option(name = "PP", value = "pp")]
    Pp,
    #[option(name = "Map ranked date", value = "ranked_date")]
    RankedDate,
    #[option(name = "Score", value = "score")]
    Score,
    #[option(name = "Stars", value = "stars")]
    Stars,
}

impl Default for ScoreOrder {
    fn default() -> Self {
        Self::Pp
    }
}
