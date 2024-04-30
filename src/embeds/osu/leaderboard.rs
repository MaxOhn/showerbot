use std::fmt::{Display, Formatter, Result as FmtResult, Write};

use command_macros::EmbedData;
use hashbrown::{hash_map::Entry, HashMap};
use rosu_pp::{Beatmap as Map, BeatmapExt, DifficultyAttributes, ScoreState};
use rosu_v2::{
    model::score::Score,
    prelude::{BeatmapExtended, BeatmapsetExtended, GameMode},
};

use crate::{
    core::Context,
    error::PpError,
    util::{
        builder::{AuthorBuilder, FooterBuilder},
        constants::{AVATAR_URL, MAP_THUMB_URL, OSU_BASE},
        datetime::HowLongAgoDynamic,
        numbers::with_comma_int,
        osu::prepare_beatmap_file,
        ScoreExt,
    },
    BotResult,
};

const UNKNOWN_NAME: &str = "<unknown name>";

#[derive(EmbedData)]
pub struct LeaderboardEmbed {
    description: String,
    thumbnail: String,
    author: AuthorBuilder,
    footer: FooterBuilder,
}

impl LeaderboardEmbed {
    #[allow(clippy::too_many_arguments)]
    pub async fn new<'i, S>(
        map: &BeatmapExtended,
        scores: Option<S>,
        author_icon: &Option<String>,
        idx: usize,
        ctx: &Context,
        pages: (usize, usize),
    ) -> BotResult<Self>
    where
        S: Iterator<Item = &'i Score>,
    {
        let BeatmapsetExtended {
            artist,
            title,
            creator_name,
            creator_id,
            ..
        } = map.mapset.as_deref().unwrap();

        let mut author_text = String::with_capacity(32);

        if map.mode == GameMode::Mania {
            let _ = write!(author_text, "[{}K] ", map.cs as u32);
        }

        let _ = write!(
            author_text,
            "{artist} - {title} [{version}] [{stars:.2}★]",
            version = map.version,
            stars = map.stars
        );

        let description = if let Some(scores) = scores {
            let map_path = prepare_beatmap_file(ctx, map.map_id).await?;
            let rosu_map = Map::from_path(map_path).await.map_err(PpError::from)?;

            let mut mod_map = HashMap::new();
            let mut description = String::with_capacity(256);
            let mut username = String::with_capacity(32);

            for (score, i) in scores.zip(idx + 1..) {
                username.clear();

                let _ = write!(
                    username,
                    "[{name}]({OSU_BASE}users/{id})",
                    name = score
                        .user
                        .as_ref()
                        .map_or(UNKNOWN_NAME, |user| user.username.as_str()),
                    id = score.user_id
                );

                let _ = writeln!(
                    description,
                    "**{i}.** {grade} **{username}**: {score} [ {combo} ] **+{mods}**\n\
                    - {pp} • {acc:.2}% • {miss}{ago}",
                    grade = score.grade_emote(map.mode),
                    score = with_comma_int(score.score),
                    combo = ComboFormatter::new(score, map),
                    mods = score.mods,
                    pp = get_pp(&mut mod_map, score, &rosu_map).await,
                    acc = score.accuracy,
                    miss = MissFormat(score.statistics.miss),
                    ago = HowLongAgoDynamic::new(&score.ended_at),
                );
            }

            description
        } else {
            "No scores found".to_string()
        };

        let mut author = AuthorBuilder::new(author_text).url(format!("{OSU_BASE}b/{}", map.map_id));

        if let Some(ref author_icon) = author_icon {
            author = author.icon_url(author_icon.to_owned());
        }

        let footer_text = format!(
            "{:?} map by {creator_name} • Page {}/{}",
            map.status, pages.0, pages.1,
        );

        let footer = FooterBuilder::new(footer_text).icon_url(format!("{AVATAR_URL}{creator_id}"));

        Ok(Self {
            author,
            description,
            footer,
            thumbnail: format!("{MAP_THUMB_URL}{}l.jpg", map.mapset_id),
        })
    }
}

async fn get_pp(
    mod_map: &mut HashMap<u32, (DifficultyAttributes, f32)>,
    score: &Score,
    map: &Map,
) -> PPFormatter {
    let bits = score.mods.bits();

    let (attrs, max_pp) = match mod_map.entry(bits) {
        Entry::Occupied(entry) => {
            let (attrs, max_pp) = entry.get();

            (attrs.to_owned(), *max_pp)
        }
        Entry::Vacant(entry) => {
            let attrs = map.max_pp(bits);
            let max_pp = attrs.pp() as f32;
            let (attrs, max_pp) = entry.insert((attrs.into(), max_pp));

            (attrs.to_owned(), *max_pp)
        }
    };

    let state = ScoreState {
        max_combo: score.max_combo as usize,
        n_geki: score.statistics.perfect as usize,
        n_katu: score.statistics.good as usize,
        n300: score.statistics.great as usize,
        n100: score.statistics.ok as usize,
        n50: score.statistics.meh as usize,
        n_misses: score.statistics.miss as usize,
    };

    let pp = map
        .pp()
        .attributes(attrs)
        .mods(score.mods.bits())
        .state(state)
        .calculate()
        .pp() as f32;

    PPFormatter(pp, max_pp)
}

struct PPFormatter(f32, f32);

impl Display for PPFormatter {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "**{:.2}**/{:.2}PP", self.0, self.1)
    }
}

struct ComboFormatter<'a> {
    score: &'a Score,
    map: &'a BeatmapExtended,
}

impl<'a> ComboFormatter<'a> {
    fn new(score: &'a Score, map: &'a BeatmapExtended) -> Self {
        Self { score, map }
    }
}

impl<'a> Display for ComboFormatter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "**{}x**", self.score.max_combo)?;

        if let Some(combo) = self.map.max_combo {
            write!(f, "/{combo}x")
        } else {
            let mut ratio = self.score.statistics.perfect as f32;

            if self.score.statistics.great > 0 {
                ratio /= self.score.statistics.great as f32
            }

            write!(f, " / {ratio:.2}")
        }
    }
}

struct MissFormat(u32);

impl Display for MissFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.0 == 0 {
            return Ok(());
        }

        write!(f, "{miss}:x: ", miss = self.0)
    }
}
