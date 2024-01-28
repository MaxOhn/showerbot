use std::sync::Arc;

use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use rosu_v2::Osu;
use twilight_gateway::{stream, CloseFrame, Config, EventTypeFlags, Intents, Shard};
use twilight_http::{client::InteractionClient, Client};
use twilight_model::{
    channel::message::AllowedMentions,
    id::{marker::ApplicationMarker, Id},
};
use twilight_standby::Standby;

use crate::{core::CONFIG, custom_client::CustomClient, BotResult, Error as BotError};

use super::{BotConfig, Cache};

mod messages;

pub struct Context {
    pub cache: Cache,
    pub http: Arc<Client>,
    pub standby: Standby,
    pub application_id: Id<ApplicationMarker>,
    pub prefixes: Box<[Box<str>]>,
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

    pub async fn new() -> BotResult<(Self, Vec<Shard>)> {
        let config = CONFIG.get().unwrap();

        // Connect to discord API
        let (http, application_id) = discord_http(config).await?;

        // Connect to osu! API
        let osu_client_id = config.tokens.osu_client_id;
        let osu_client_secret = &config.tokens.osu_client_secret;
        let osu = Osu::new(osu_client_id, osu_client_secret).await?;

        // Log custom client into osu!
        let custom = CustomClient::new(config).await?;

        let cache = Cache::new().await;

        let clients = Clients::new(osu, custom);

        let shards = discord_gateway(config, &http).await?;

        let ctx = Self {
            cache,
            http,
            clients,
            application_id,
            standby: Standby::new(),
            prefixes: config.prefixes.clone(),
        };

        info!("Prefixes: {:?}", ctx.prefixes);

        Ok((ctx, shards))
    }

    pub async fn down(shards: &mut [Shard]) {
        shards
            .iter_mut()
            .map(|shard| {
                let shard_id = shard.id().number();

                shard
                    .close(CloseFrame::NORMAL)
                    .map(move |res| (shard_id, res))
            })
            .collect::<FuturesUnordered<_>>()
            .map(|(shard_id, res)| match res {
                Ok(_) => {}
                Err(err) => warn!(shard_id, ?err, "Failed to close shard"),
            })
            .collect()
            .await
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

async fn discord_http(config: &BotConfig) -> BotResult<(Arc<Client>, Id<ApplicationMarker>)> {
    let mentions = AllowedMentions {
        replied_user: true,
        ..Default::default()
    };

    // Connect to the discord http client
    let http = Client::builder()
        .token(config.tokens.discord.to_string())
        .remember_invalid_token(false)
        .default_allowed_mentions(mentions)
        .build();

    let http = Arc::new(http);

    let current_user = http.current_user().await?.model().await?;

    let application_id = current_user.id.cast();

    info!(
        "Connecting to Discord as {}#{:04}...",
        current_user.name, current_user.discriminator
    );

    Ok((http, application_id))
}

async fn discord_gateway(config: &BotConfig, http: &Client) -> BotResult<Vec<Shard>> {
    let intents = Intents::GUILDS
        | Intents::GUILD_MEMBERS
        | Intents::GUILD_MESSAGES
        | Intents::GUILD_MESSAGE_REACTIONS
        | Intents::DIRECT_MESSAGES
        | Intents::DIRECT_MESSAGE_REACTIONS
        | Intents::MESSAGE_CONTENT;

    let event_types = EventTypeFlags::CHANNEL_CREATE
        | EventTypeFlags::CHANNEL_DELETE
        | EventTypeFlags::CHANNEL_UPDATE
        | EventTypeFlags::GUILD_CREATE
        | EventTypeFlags::GUILD_DELETE
        | EventTypeFlags::GUILD_UPDATE
        | EventTypeFlags::INTERACTION_CREATE
        | EventTypeFlags::MEMBER_ADD
        | EventTypeFlags::MEMBER_REMOVE
        | EventTypeFlags::MEMBER_UPDATE
        | EventTypeFlags::MEMBER_CHUNK
        | EventTypeFlags::MESSAGE_CREATE
        | EventTypeFlags::MESSAGE_DELETE
        | EventTypeFlags::MESSAGE_DELETE_BULK
        | EventTypeFlags::READY
        | EventTypeFlags::ROLE_CREATE
        | EventTypeFlags::ROLE_DELETE
        | EventTypeFlags::ROLE_UPDATE
        | EventTypeFlags::THREAD_CREATE
        | EventTypeFlags::THREAD_DELETE
        | EventTypeFlags::THREAD_UPDATE
        | EventTypeFlags::UNAVAILABLE_GUILD
        | EventTypeFlags::USER_UPDATE;

    let config = Config::builder(config.tokens.discord.to_string(), intents)
        .event_types(event_types)
        .build();

    stream::create_recommended(http, config, |_, builder| builder.build())
        .await
        .map(Iterator::collect)
        .map_err(BotError::StartRecommended)
}
