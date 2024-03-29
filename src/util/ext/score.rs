use crate::{
    custom_client::ScraperScore,
    util::{numbers::round, osu::grade_emote},
};

use rosu_v2::prelude::{GameModIntermode, GameMode, GameMods, Grade, Score};
use std::fmt::Write;

pub trait ScoreExt: Send + Sync {
    // Required to implement
    fn count_miss(&self) -> u32;
    fn count_50(&self) -> u32;
    fn count_100(&self) -> u32;
    fn count_300(&self) -> u32;
    fn count_geki(&self) -> u32;
    fn count_katu(&self) -> u32;
    fn max_combo(&self) -> u32;
    fn mods(&self) -> &GameMods;
    fn score(&self) -> u32;
    fn pp(&self) -> Option<f32>;
    fn acc(&self, mode: GameMode) -> f32;

    // Optional to implement
    fn grade(&self, mode: GameMode) -> Grade {
        match mode {
            GameMode::Osu => self.osu_grade(),
            GameMode::Mania => self.mania_grade(Some(self.acc(GameMode::Mania))),
            GameMode::Catch => self.ctb_grade(Some(self.acc(GameMode::Catch))),
            GameMode::Taiko => self.taiko_grade(),
        }
    }
    fn hits(&self, mode: u8) -> u32 {
        let mut amount = self.count_300() + self.count_100() + self.count_miss();

        if mode != 1 {
            // TKO
            amount += self.count_50();

            if mode != 0 {
                // STD
                amount += self.count_katu();

                // CTB
                amount += (mode != 2) as u32 * self.count_geki();
            }
        }

        amount
    }

    // Processing to strings
    fn grade_emote(&self, mode: GameMode) -> &'static str {
        grade_emote(self.grade(mode))
    }
    fn hits_string(&self, mode: GameMode) -> String {
        let mut hits = String::from("{");
        if mode == GameMode::Mania {
            let _ = write!(hits, "{}/", self.count_geki());
        }
        let _ = write!(hits, "{}/", self.count_300());
        if mode == GameMode::Mania {
            let _ = write!(hits, "{}/", self.count_katu());
        }
        let _ = write!(hits, "{}/", self.count_100());
        if mode != GameMode::Taiko {
            let _ = write!(hits, "{}/", self.count_50());
        }
        let _ = write!(hits, "{}}}", self.count_miss());
        hits
    }

    // #########################
    // ## Auxiliary functions ##
    // #########################
    fn osu_grade(&self) -> Grade {
        let passed_objects = self.hits(GameMode::Osu as u8);
        let mods = self.mods();

        if self.count_300() == passed_objects {
            return if mods.contains_intermode(GameModIntermode::Hidden)
                || mods.contains_intermode(GameModIntermode::Flashlight)
            {
                Grade::XH
            } else {
                Grade::X
            };
        }

        let ratio300 = self.count_300() as f32 / passed_objects as f32;
        let ratio50 = self.count_50() as f32 / passed_objects as f32;

        if ratio300 > 0.9 && ratio50 < 0.01 && self.count_miss() == 0 {
            if mods.contains_intermode(GameModIntermode::Hidden)
                || mods.contains_intermode(GameModIntermode::Flashlight)
            {
                Grade::SH
            } else {
                Grade::S
            }
        } else if ratio300 > 0.9 || (ratio300 > 0.8 && self.count_miss() == 0) {
            Grade::A
        } else if ratio300 > 0.8 || (ratio300 > 0.7 && self.count_miss() == 0) {
            Grade::B
        } else if ratio300 > 0.6 {
            Grade::C
        } else {
            Grade::D
        }
    }

    fn mania_grade(&self, acc: Option<f32>) -> Grade {
        let passed_objects = self.hits(GameMode::Mania as u8);
        let mods = self.mods();

        if self.count_geki() == passed_objects {
            return if mods.contains_intermode(GameModIntermode::Hidden)
                || mods.contains_intermode(GameModIntermode::Flashlight)
            {
                Grade::XH
            } else {
                Grade::X
            };
        }

        let acc = acc.unwrap_or_else(|| self.acc(GameMode::Mania));

        if acc > 95.0 {
            if mods.contains_intermode(GameModIntermode::Hidden)
                || mods.contains_intermode(GameModIntermode::Flashlight)
            {
                Grade::SH
            } else {
                Grade::S
            }
        } else if acc > 90.0 {
            Grade::A
        } else if acc > 80.0 {
            Grade::B
        } else if acc > 70.0 {
            Grade::C
        } else {
            Grade::D
        }
    }

    fn taiko_grade(&self) -> Grade {
        let mods = self.mods();
        let passed_objects = self.hits(GameMode::Taiko as u8);
        let count_300 = self.count_300();

        if count_300 == passed_objects {
            return if mods.contains_intermode(GameModIntermode::Hidden)
                || mods.contains_intermode(GameModIntermode::Flashlight)
            {
                Grade::XH
            } else {
                Grade::X
            };
        }

        let ratio300 = count_300 as f32 / passed_objects as f32;
        let count_miss = self.count_miss();

        if ratio300 > 0.9 && count_miss == 0 {
            if mods.contains_intermode(GameModIntermode::Hidden)
                || mods.contains_intermode(GameModIntermode::Flashlight)
            {
                Grade::SH
            } else {
                Grade::S
            }
        } else if ratio300 > 0.9 || (ratio300 > 0.8 && count_miss == 0) {
            Grade::A
        } else if ratio300 > 0.8 || (ratio300 > 0.7 && count_miss == 0) {
            Grade::B
        } else if ratio300 > 0.6 {
            Grade::C
        } else {
            Grade::D
        }
    }

    fn ctb_grade(&self, acc: Option<f32>) -> Grade {
        let mods = self.mods();
        let acc = acc.unwrap_or_else(|| self.acc(GameMode::Catch));

        if (100.0 - acc).abs() <= std::f32::EPSILON {
            if mods.contains_intermode(GameModIntermode::Hidden)
                || mods.contains_intermode(GameModIntermode::Flashlight)
            {
                Grade::XH
            } else {
                Grade::X
            }
        } else if acc > 98.0 {
            if mods.contains_intermode(GameModIntermode::Hidden)
                || mods.contains_intermode(GameModIntermode::Flashlight)
            {
                Grade::SH
            } else {
                Grade::S
            }
        } else if acc > 94.0 {
            Grade::A
        } else if acc > 90.0 {
            Grade::B
        } else if acc > 85.0 {
            Grade::C
        } else {
            Grade::D
        }
    }
}

// #####################
// ## Implementations ##
// #####################

impl ScoreExt for Score {
    fn count_miss(&self) -> u32 {
        self.statistics.count_miss
    }
    fn count_50(&self) -> u32 {
        self.statistics.count_50
    }
    fn count_100(&self) -> u32 {
        self.statistics.count_100
    }
    fn count_300(&self) -> u32 {
        self.statistics.count_300
    }
    fn count_geki(&self) -> u32 {
        self.statistics.count_geki
    }
    fn count_katu(&self) -> u32 {
        self.statistics.count_katu
    }
    fn max_combo(&self) -> u32 {
        self.max_combo
    }
    fn mods(&self) -> &GameMods {
        &self.mods
    }
    fn grade(&self, _mode: GameMode) -> Grade {
        self.grade
    }
    fn score(&self) -> u32 {
        self.score
    }
    fn pp(&self) -> Option<f32> {
        self.pp
    }
    fn acc(&self, _: GameMode) -> f32 {
        round(self.accuracy)
    }
}

impl ScoreExt for ScraperScore {
    fn count_miss(&self) -> u32 {
        self.count_miss
    }
    fn count_50(&self) -> u32 {
        self.count50
    }
    fn count_100(&self) -> u32 {
        self.count100
    }
    fn count_300(&self) -> u32 {
        self.count300
    }
    fn count_geki(&self) -> u32 {
        self.count_geki
    }
    fn count_katu(&self) -> u32 {
        self.count_katu
    }
    fn max_combo(&self) -> u32 {
        self.max_combo
    }
    fn mods(&self) -> &GameMods {
        &self.mods
    }
    fn grade(&self, _: GameMode) -> Grade {
        self.grade
    }
    fn score(&self) -> u32 {
        self.score
    }
    fn pp(&self) -> Option<f32> {
        self.pp
    }
    fn acc(&self, _: GameMode) -> f32 {
        self.accuracy
    }
}
