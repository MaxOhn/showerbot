use eyre::Report;
use twilight_model::id::{marker::GuildMarker, Id};

use crate::{
    core::commands::prefix::Stream,
    database::{GuildConfig, Prefix, Prefixes},
    BotResult, Context,
};

impl Context {
    async fn guild_config_<'g, F, O>(&self, guild_id: Id<GuildMarker>, f: F) -> O
    where
        F: FnOnce(&GuildConfig) -> O,
    {
        if let Some(config) = self.data.guilds.pin().get(&guild_id) {
            return f(config);
        }

        let config = GuildConfig::default();

        if let Err(err) = self.psql().upsert_guild_config(guild_id, &config).await {
            let wrap = format!("failed to insert guild {guild_id}");
            let report = Report::new(err).wrap_err(wrap);
            warn!("{report:?}");
        }

        let res = f(&config);
        self.data.guilds.pin().insert(guild_id, config);

        res
    }

    pub async fn guild_prefixes(&self, guild_id: Id<GuildMarker>) -> Prefixes {
        self.guild_config_(guild_id, |config| config.prefixes.clone())
            .await
    }

    pub async fn guild_prefixes_find(
        &self,
        guild_id: Id<GuildMarker>,
        stream: &Stream<'_>,
    ) -> Option<Prefix> {
        let f = |config: &GuildConfig| {
            config
                .prefixes
                .iter()
                .find(|p| stream.starts_with(p))
                .cloned()
        };

        self.guild_config_(guild_id, f).await
    }

    pub async fn guild_first_prefix(&self, guild_id: Option<Id<GuildMarker>>) -> Prefix {
        match guild_id {
            Some(guild_id) => {
                self.guild_config_(guild_id, |config| config.prefixes[0].clone())
                    .await
            }
            None => "<".into(),
        }
    }

    pub async fn update_guild_config<F>(&self, guild_id: Id<GuildMarker>, f: F) -> BotResult<()>
    where
        F: FnOnce(&mut GuildConfig),
    {
        let guilds = &self.data.guilds;

        let mut config = guilds
            .pin()
            .get(&guild_id)
            .map(GuildConfig::to_owned)
            .unwrap_or_default();

        f(&mut config);
        self.psql().upsert_guild_config(guild_id, &config).await?;
        guilds.pin().insert(guild_id, config);

        Ok(())
    }
}
