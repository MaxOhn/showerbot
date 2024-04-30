use rosu_v2::{
    model::score::ScoreStatistics,
    prelude::{CountryCode, GameMode, GameMods, Grade, ModeAsSeed, RankStatus},
};
use serde::{
    de::{DeserializeSeed, Error as DeError},
    Deserialize, Deserializer,
};
use serde_json::value::RawValue;
use time::OffsetDateTime;

use super::deser;

#[derive(Deserialize)]
pub struct ScraperScores {
    scores: Vec<ScraperScore>,
}

impl ScraperScores {
    pub fn get(self) -> Vec<ScraperScore> {
        self.scores
    }
}

pub struct ScraperScore {
    pub id: u64,
    pub user_id: u32,
    pub username: String,
    pub country_code: CountryCode,
    pub accuracy: f32,
    pub mode: GameMode,
    pub mods: GameMods,
    pub score: u32,
    pub max_combo: u32,
    // pub perfect: bool,
    pub pp: Option<f32>,
    pub grade: Grade,
    pub date: OffsetDateTime,
    pub replay: bool,
    pub count50: u32,
    pub count100: u32,
    pub count300: u32,
    pub count_geki: u32,
    pub count_katu: u32,
    pub count_miss: u32,
}

impl<'de> Deserialize<'de> for ScraperScore {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Outer<'mods> {
            id: u64,
            user_id: u32,
            #[serde(with = "deser::adjust_acc")]
            accuracy: f32,
            #[serde(borrow)]
            mods: Option<&'mods RawValue>,
            #[serde(rename = "total_score")]
            score: u32,
            #[serde(rename = "ruleset_id")]
            mode: GameMode,
            max_combo: u32,
            statistics: ScoreStatistics,
            pp: Option<f32>,
            rank: Grade,
            #[serde(with = "deser::datetime")]
            ended_at: OffsetDateTime,
            replay: bool,
            user: ScraperUser,
        }

        #[derive(Deserialize)]
        pub struct ScraperUser {
            username: String,
            country_code: CountryCode,
        }

        let helper = Outer::deserialize(d)?;

        let mods = match helper.mods {
            Some(raw) => {
                let mut d = serde_json::Deserializer::from_str(raw.get());

                ModeAsSeed::<GameMods>::new(helper.mode)
                    .deserialize(&mut d)
                    .map_err(DeError::custom)?
            }
            None => GameMods::default(),
        };

        Ok(ScraperScore {
            id: helper.id,
            user_id: helper.user_id,
            username: helper.user.username,
            country_code: helper.user.country_code,
            accuracy: helper.accuracy,
            mode: helper.mode,
            mods,
            score: helper.score,
            max_combo: helper.max_combo,
            pp: helper.pp,
            grade: helper.rank,
            date: helper.ended_at,
            replay: helper.replay,
            count50: helper.statistics.meh,
            count100: helper.statistics.ok,
            count300: helper.statistics.great,
            count_geki: helper.statistics.perfect,
            count_katu: helper.statistics.good,
            count_miss: helper.statistics.miss,
        })
    }
}

#[derive(Deserialize)]
pub struct ScraperBeatmap {
    pub id: u32,
    pub beatmapset_id: u32,
    #[serde(rename = "mode_int")]
    pub mode: GameMode,
    pub difficulty_rating: f32,
    pub version: String,
    pub total_length: u32,
    pub hit_length: u32,
    pub bpm: f32,
    pub cs: f32,
    #[serde(rename = "drain")]
    pub hp: f32,
    #[serde(rename = "accuracy")]
    pub od: f32,
    pub ar: f32,
    #[serde(default)]
    pub playcount: u32,
    #[serde(default)]
    pub passcount: u32,
    #[serde(default)]
    pub count_circles: u32,
    #[serde(default)]
    pub count_sliders: u32,
    #[serde(default)]
    pub count_spinner: u32,
    #[serde(default)]
    pub count_total: u32,
    #[serde(with = "deser::datetime")]
    pub last_updated: OffsetDateTime,
    pub ranked: RankStatus,
}
