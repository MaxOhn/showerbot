use std::path::PathBuf;

use rosu_v2::prelude::{GameMode, GameMods, Grade, Score};
use time::OffsetDateTime;
use tokio::{fs::File, io::AsyncWriteExt};
use twilight_model::channel::{message::embed::Embed, Message};

use crate::{
    core::Context,
    error::MapFileError,
    util::{constants::OSU_BASE, matcher},
    CONFIG,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ModSelection {
    Include(GameMods),
    Exclude(GameMods),
    Exact(GameMods),
}

#[allow(dead_code)]
pub fn flag_url_svg(country_code: &str) -> String {
    assert_eq!(
        country_code.len(),
        2,
        "country code `{country_code}` is invalid",
    );

    const OFFSET: u32 = 0x1F1A5;
    let bytes = country_code.as_bytes();

    let url = format!(
        "{OSU_BASE}assets/images/flags/{:x}-{:x}.svg",
        bytes[0].to_ascii_uppercase() as u32 + OFFSET,
        bytes[1].to_ascii_uppercase() as u32 + OFFSET
    );

    url
}

pub fn grade_emote(grade: Grade) -> &'static str {
    CONFIG.get().unwrap().grade(grade)
}

pub async fn prepare_beatmap_file(ctx: &Context, map_id: u32) -> Result<PathBuf, MapFileError> {
    let mut map_path = CONFIG.get().unwrap().paths.maps.clone();
    map_path.push(format!("{map_id}.osu"));

    if !map_path.exists() {
        let bytes = ctx.client().get_map_file(map_id).await?;
        let mut file = File::create(&map_path).await?;
        file.write_all(&bytes).await?;
        info!("Downloaded {map_id}.osu successfully");
    }

    Ok(map_path)
}

pub trait ExtractablePp {
    fn extract_pp(&self) -> Vec<f32>;
}

impl ExtractablePp for [Score] {
    fn extract_pp(&self) -> Vec<f32> {
        self.iter().map(|s| s.pp.unwrap_or(0.0)).collect()
    }
}

pub trait PpListUtil {
    fn accum_weighted(&self) -> f32;
}

impl PpListUtil for [f32] {
    fn accum_weighted(&self) -> f32 {
        self.iter()
            .copied()
            .zip(0..)
            .fold(0.0, |sum, (pp, i)| sum + pp * 0.95_f32.powi(i))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MapIdType {
    Map(u32),
    Set(u32),
}

impl MapIdType {
    /// Looks for map or mapset id
    pub fn from_msg(msg: &Message) -> Option<Self> {
        if msg.content.chars().all(|c| c.is_numeric()) {
            return Self::from_embeds(&msg.embeds);
        }

        matcher::get_osu_map_id(&msg.content)
            .map(Self::Map)
            .or_else(|| matcher::get_osu_mapset_id(&msg.content).map(Self::Set))
            .or_else(|| Self::from_embeds(&msg.embeds))
    }

    /// Looks for map or mapset id
    pub fn from_embeds(embeds: &[Embed]) -> Option<Self> {
        embeds.iter().find_map(|embed| {
            let url = embed
                .author
                .as_ref()
                .and_then(|author| author.url.as_deref());

            url.and_then(matcher::get_osu_map_id)
                .map(Self::Map)
                .or_else(|| url.and_then(matcher::get_osu_mapset_id).map(Self::Set))
                .or_else(|| {
                    embed
                        .url
                        .as_deref()
                        .and_then(matcher::get_osu_map_id)
                        .map(Self::Map)
                })
                .or_else(|| {
                    embed
                        .url
                        .as_deref()
                        .and_then(matcher::get_osu_mapset_id)
                        .map(Self::Set)
                })
        })
    }

    /// Only looks for map id
    pub fn map_from_msgs(msgs: &[Message], idx: usize) -> Option<u32> {
        msgs.iter().filter_map(Self::map_from_msg).nth(idx)
    }

    /// Only looks for map id
    pub fn map_from_msg(msg: &Message) -> Option<u32> {
        if msg.content.chars().all(|c| c.is_numeric()) {
            return Self::map_from_embeds(&msg.embeds);
        }

        matcher::get_osu_map_id(&msg.content).or_else(|| Self::map_from_embeds(&msg.embeds))
    }

    /// Only looks for map id
    pub fn map_from_embeds(embeds: &[Embed]) -> Option<u32> {
        embeds.iter().find_map(|embed| {
            embed
                .author
                .as_ref()
                .and_then(|author| author.url.as_deref())
                .and_then(matcher::get_osu_map_id)
                .or_else(|| embed.url.as_deref().and_then(matcher::get_osu_map_id))
        })
    }
}

pub trait SortableScore {
    fn acc(&self) -> f32;
    fn bpm(&self) -> f32;
    fn created_at(&self) -> OffsetDateTime;
    fn map_id(&self) -> u32;
    fn mapset_id(&self) -> u32;
    fn max_combo(&self) -> u32;
    fn mode(&self) -> GameMode;
    fn mods(&self) -> GameMods;
    fn n_misses(&self) -> u32;
    fn pp(&self) -> Option<f32>;
    fn score(&self) -> u32;
    fn score_id(&self) -> Option<u64>;
    fn seconds_drain(&self) -> u32;
    fn stars(&self) -> f32;
    fn total_hits_sort(&self) -> u32;
}

impl SortableScore for Score {
    fn acc(&self) -> f32 {
        self.accuracy
    }

    fn bpm(&self) -> f32 {
        self.map.as_ref().map_or(0.0, |map| map.bpm)
    }

    fn created_at(&self) -> OffsetDateTime {
        self.ended_at
    }

    fn map_id(&self) -> u32 {
        self.map.as_ref().map_or(0, |map| map.map_id)
    }

    fn mapset_id(&self) -> u32 {
        self.mapset.as_ref().map_or(0, |mapset| mapset.mapset_id)
    }

    fn max_combo(&self) -> u32 {
        self.max_combo
    }

    fn mode(&self) -> GameMode {
        self.mode
    }

    fn mods(&self) -> GameMods {
        self.mods
    }

    fn n_misses(&self) -> u32 {
        self.statistics.count_miss
    }

    fn pp(&self) -> Option<f32> {
        self.pp
    }

    fn score(&self) -> u32 {
        self.score
    }

    fn score_id(&self) -> Option<u64> {
        self.score_id
    }

    fn seconds_drain(&self) -> u32 {
        self.map.as_ref().map_or(0, |map| map.seconds_drain)
    }

    fn stars(&self) -> f32 {
        self.map.as_ref().map_or(0.0, |map| map.stars)
    }

    fn total_hits_sort(&self) -> u32 {
        self.total_hits()
    }
}

macro_rules! impl_sortable_score_tuple {
    (($($ty:ty),*) => $idx:tt) => {
        impl SortableScore for ($($ty),*) {
            fn acc(&self) -> f32 {
                SortableScore::acc(&self.$idx)
            }

            fn bpm(&self) -> f32 {
                SortableScore::bpm(&self.$idx)
            }

            fn created_at(&self) -> OffsetDateTime {
                SortableScore::created_at(&self.$idx)
            }

            fn map_id(&self) -> u32 {
                SortableScore::map_id(&self.$idx)
            }

            fn mapset_id(&self) -> u32 {
                SortableScore::mapset_id(&self.$idx)
            }

            fn max_combo(&self) -> u32 {
                SortableScore::max_combo(&self.$idx)
            }

            fn mode(&self) -> GameMode {
                SortableScore::mode(&self.$idx)
            }

            fn mods(&self) -> GameMods {
                SortableScore::mods(&self.$idx)
            }

            fn n_misses(&self) -> u32 {
                SortableScore::n_misses(&self.$idx)
            }

            fn pp(&self) -> Option<f32> {
                SortableScore::pp(&self.$idx)
            }

            fn score(&self) -> u32 {
                SortableScore::score(&self.$idx)
            }

            fn score_id(&self) -> Option<u64> {
                SortableScore::score_id(&self.$idx)
            }

            fn seconds_drain(&self) -> u32 {
                SortableScore::seconds_drain(&self.$idx)
            }

            fn stars(&self) -> f32 {
                SortableScore::stars(&self.1)
            }

            fn total_hits_sort(&self) -> u32 {
                SortableScore::total_hits_sort(&self.$idx)
            }
        }
    };
}

impl_sortable_score_tuple!((usize, Score) => 1);
impl_sortable_score_tuple!((usize, Score, Option<f32>) => 1);
