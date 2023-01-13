use std::str::FromStr;

use rosu_v2::prelude::{GameMode, Grade};
use twilight_interactions::command::{CommandOption, CreateOption};

pub mod help;
pub mod osu;
pub mod utility;

#[derive(Copy, Clone, CommandOption, CreateOption, Eq, PartialEq)]
pub enum ShowHideOption {
    #[option(name = "Show", value = "show")]
    Show,
    #[option(name = "Hide", value = "hide")]
    Hide,
}

#[derive(Copy, Clone, CommandOption, CreateOption, Eq, PartialEq)]
pub enum EnableDisable {
    #[option(name = "Enable", value = "enable")]
    Enable,
    #[option(name = "Disable", value = "disable")]
    Disable,
}

#[derive(CommandOption, CreateOption)]
pub enum ThreadChannel {
    #[option(name = "Stay in channel", value = "channel")]
    Channel,
    #[option(name = "Start new thread", value = "thread")]
    Thread,
}

#[derive(Copy, Clone, CommandOption, CreateOption)]
pub enum GameModeOption {
    #[option(name = "osu", value = "osu")]
    Osu,
    #[option(name = "taiko", value = "taiko")]
    Taiko,
    #[option(name = "ctb", value = "ctb")]
    Catch,
    #[option(name = "mania", value = "mania")]
    Mania,
}

impl From<GameModeOption> for GameMode {
    #[inline]
    fn from(mode: GameModeOption) -> Self {
        match mode {
            GameModeOption::Osu => Self::Osu,
            GameModeOption::Taiko => Self::Taiko,
            GameModeOption::Catch => Self::Catch,
            GameModeOption::Mania => Self::Mania,
        }
    }
}

impl From<GameMode> for GameModeOption {
    #[inline]
    fn from(mode: GameMode) -> Self {
        match mode {
            GameMode::Osu => Self::Osu,
            GameMode::Taiko => Self::Taiko,
            GameMode::Catch => Self::Catch,
            GameMode::Mania => Self::Mania,
        }
    }
}

#[derive(CommandOption, CreateOption)]
pub enum GradeOption {
    #[option(name = "SS", value = "ss")]
    SS,
    #[option(name = "S", value = "s")]
    S,
    #[option(name = "A", value = "a")]
    A,
    #[option(name = "B", value = "b")]
    B,
    #[option(name = "C", value = "c")]
    C,
    #[option(name = "D", value = "d")]
    D,
    #[option(name = "F", value = "f")]
    F,
}

impl From<GradeOption> for Grade {
    #[inline]
    fn from(grade: GradeOption) -> Self {
        match grade {
            GradeOption::SS => Self::X,
            GradeOption::S => Self::S,
            GradeOption::A => Self::A,
            GradeOption::B => Self::B,
            GradeOption::C => Self::C,
            GradeOption::D => Self::D,
            GradeOption::F => Self::F,
        }
    }
}

impl FromStr for GradeOption {
    type Err = &'static str;

    // ! Make sure the given strings are lower case
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let grade = match s {
            "x" | "ss" => Self::SS,
            "s" => Self::S,
            "a" => Self::A,
            "b" => Self::B,
            "c" => Self::C,
            "d" => Self::D,
            "f" => Self::F,
            _ => {
                return Err("Failed to parse `grade`.\n\
                Valid grades are: `SS`, `S`, `A`, `B`, `C`, `D`, or `F`")
            }
        };

        Ok(grade)
    }
}
