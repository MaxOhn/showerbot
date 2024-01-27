use std::sync::Arc;

use rosu_v2::Osu;
use twilight_gateway::{cluster::Events, Cluster};
use twilight_http::{client::InteractionClient, Client};
use twilight_model::{
    channel::message::allowed_mentions::AllowedMentionsBuilder,
    id::{marker::ApplicationMarker, Id},
};
use twilight_standby::Standby;

use crate::{core::CONFIG, custom_client::CustomClient, BotResult};

use super::{cluster::build_cluster, Cache};

mod messages;

pub struct Context {
    pub cache: Cache,
    pub cluster: Cluster,
    pub http: Arc<Client>,
    pub standby: Standby,
    pub application_id: Id<ApplicationMarker>,
    clients: Clients,
}

impl Context {
    pub fn interaction(&self) -> InteractionClient<'_> {
        self.http.interaction(self.application_id)
    }

    pub fn osu(&self) -> &Osu {
        &self.clients.osu
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

        // Connect to osu! API
        let osu_client_id = config.tokens.osu_client_id;
        let osu_client_secret = &config.tokens.osu_client_secret;
        let osu = Osu::new(osu_client_id, osu_client_secret).await?;

        // Log custom client into osu!
        let custom = CustomClient::new(config).await?;

        let (cache, resume_data) = Cache::new().await;

        let clients = Clients::new(osu, custom);
        let (cluster, events) =
            build_cluster(discord_token, Arc::clone(&http), resume_data).await?;

        let ctx = Self {
            cache,
            http,
            clients,
            cluster,
            application_id,
            standby: Standby::new(),
        };

        Ok((ctx, events))
    }
}

struct Clients {
    custom: CustomClient,
    osu: Osu,
}

impl Clients {
    fn new(osu: Osu, custom: CustomClient) -> Self {
        Self { osu, custom }
    }
}
