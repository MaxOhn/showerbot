use std::sync::Arc;

use dashmap::DashSet;
use flurry::HashMap as FlurryMap;
use rosu_v2::Osu;
use twilight_gateway::{cluster::Events, Cluster};
use twilight_http::{client::InteractionClient, Client};
use twilight_model::{
    channel::message::allowed_mentions::AllowedMentionsBuilder,
    id::{
        marker::{ApplicationMarker, GuildMarker, MessageMarker},
        Id,
    },
};
use twilight_standby::Standby;

use crate::{
    core::CONFIG,
    custom_client::CustomClient,
    database::{Database, GuildConfig},
    BotResult,
};

use super::{cluster::build_cluster, Cache};

mod configs;
mod messages;

pub struct Context {
    pub cache: Cache,
    pub cluster: Cluster,
    pub http: Arc<Client>,
    pub standby: Standby,
    // private to avoid deadlocks by messing up references
    data: ContextData,
    clients: Clients,
}

impl Context {
    pub fn interaction(&self) -> InteractionClient<'_> {
        self.http.interaction(self.data.application_id)
    }

    pub fn osu(&self) -> &Osu {
        &self.clients.osu
    }

    pub fn psql(&self) -> &Database {
        &self.clients.psql
    }

    /// Returns the custom client
    pub fn client(&self) -> &CustomClient {
        &self.clients.custom
    }

    pub async fn new() -> BotResult<(Self, Events)> {
        let config = CONFIG.get().unwrap();
        let discord_token = &config.tokens.discord;

        let mentions = AllowedMentionsBuilder::new()
            .replied_user()
            .roles()
            .users()
            .build();

        // Connect to the discord http client
        let http = Client::builder()
            .token(discord_token.to_owned())
            .remember_invalid_token(false)
            .default_allowed_mentions(mentions)
            .build();

        let http = Arc::new(http);

        let current_user = http.current_user().exec().await?.model().await?;
        let application_id = current_user.id.cast();

        info!(
            "Connecting to Discord as {}#{}...",
            current_user.name, current_user.discriminator
        );

        // Connect to psql database
        let psql = Database::new(&config.database_url)?;

        // Connect to osu! API
        let osu_client_id = config.tokens.osu_client_id;
        let osu_client_secret = &config.tokens.osu_client_secret;
        let osu = Osu::new(osu_client_id, osu_client_secret).await?;

        // Log custom client into osu!
        let custom = CustomClient::new(config).await?;

        let data = ContextData::new(&psql, application_id).await?;
        let (cache, resume_data) = Cache::new().await;

        let clients = Clients::new(psql, osu, custom);
        let (cluster, events) =
            build_cluster(discord_token, Arc::clone(&http), resume_data).await?;

        let ctx = Self {
            cache,
            http,
            clients,
            cluster,
            data,
            standby: Standby::new(),
        };

        Ok((ctx, events))
    }
}

struct Clients {
    custom: CustomClient,
    osu: Osu,
    psql: Database,
}

impl Clients {
    fn new(psql: Database, osu: Osu, custom: CustomClient) -> Self {
        Self { psql, osu, custom }
    }
}

struct ContextData {
    application_id: Id<ApplicationMarker>,
    guilds: FlurryMap<Id<GuildMarker>, GuildConfig>, // read-heavy
    msgs_to_process: DashSet<Id<MessageMarker>>,
}

impl ContextData {
    async fn new(psql: &Database, application_id: Id<ApplicationMarker>) -> BotResult<Self> {
        Ok(Self {
            application_id,
            guilds: psql.get_guilds().await?,
            msgs_to_process: DashSet::new(),
        })
    }
}
