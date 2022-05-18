use flurry::HashMap as FlurryMap;
use futures::stream::StreamExt;
use twilight_model::id::{marker::GuildMarker, Id};

use crate::{
    database::{Database, GuildConfig},
    BotResult,
};

impl Database {
    #[cold]
    pub async fn get_guilds(&self) -> BotResult<FlurryMap<Id<GuildMarker>, GuildConfig>> {
        let mut stream = sqlx::query!("SELECT * FROM guild_configs").fetch(&self.pool);
        let guilds = FlurryMap::with_capacity(10_000);

        {
            let gref = guilds.pin();

            while let Some(entry) = stream.next().await.transpose()? {
                let config = GuildConfig {
                    prefixes: serde_json::from_value(entry.prefixes)?,
                };

                gref.insert(Id::new(entry.guild_id as u64), config);
            }
        }

        Ok(guilds)
    }

    pub async fn upsert_guild_config(
        &self,
        guild_id: Id<GuildMarker>,
        config: &GuildConfig,
    ) -> BotResult<()> {
        let query = sqlx::query!(
            "INSERT INTO guild_configs (\
                guild_id,\
                prefixes\
            )\
            VALUES ($1,$2) ON CONFLICT (guild_id) DO \
            UPDATE \
            SET prefixes=$2",
            guild_id.get() as i64,
            serde_json::to_value(&config.prefixes)?,
        );

        query.execute(&self.pool).await?;
        info!("Inserted GuildConfig for guild {guild_id} into DB");

        Ok(())
    }
}
