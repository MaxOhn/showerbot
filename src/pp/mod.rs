use rosu_pp::{Beatmap, BeatmapExt as rosu_v2BeatmapExt, DifficultyAttributes};
use rosu_v2::model::mods::GameModsIntermode;

use crate::{core::Context, error::PpError, util::osu::prepare_beatmap_file};

enum ScoreKind {
    Mods(GameModsIntermode),
}

impl ScoreKind {
    fn mods(&self) -> u32 {
        match self {
            Self::Mods(mods) => mods.bits(),
        }
    }
}

pub struct PpCalculator {
    map: Beatmap,
    score: Option<ScoreKind>,
    difficulty: Option<DifficultyAttributes>,
}

impl PpCalculator {
    pub async fn new(ctx: &Context, map_id: u32) -> Result<PpCalculator, PpError> {
        let map_path = prepare_beatmap_file(ctx, map_id).await?;
        let map = Beatmap::from_path(map_path).await?;

        Ok(Self {
            map,
            score: None,
            difficulty: None,
        })
    }

    pub fn mods(&mut self, mods: GameModsIntermode) -> &mut Self {
        self.score = Some(ScoreKind::Mods(mods));
        self.difficulty = None;

        self
    }

    pub fn stars(&mut self) -> f64 {
        let mods = self.score.as_ref().map(ScoreKind::mods).unwrap_or_default();

        let difficulty = &mut self.difficulty;
        let map = &self.map;

        difficulty
            .get_or_insert_with(|| map.stars().mods(mods).calculate())
            .stars()
    }
}
