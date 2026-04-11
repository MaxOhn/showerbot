use crate::util::osu::ModSelection;

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
