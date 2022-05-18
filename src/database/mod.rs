mod impls;
mod models;

use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::BotResult;

pub use self::models::{DBBeatmap, DBBeatmapset, GuildConfig, Prefix, Prefixes};

pub struct Database {
    pool: PgPool,
}

impl Database {
    #[cold]
    pub fn new(uri: &str) -> BotResult<Self> {
        let pool = PgPoolOptions::new().connect_lazy(uri)?;

        Ok(Self { pool })
    }
}
