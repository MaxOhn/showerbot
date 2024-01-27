use std::{
    fmt::{Display, Error, Formatter},
    sync::Arc,
};

use eyre::Context as EyreContext;
use futures::StreamExt;
use twilight_gateway::{cluster::Events, Event};

use crate::{util::Authored, BotResult};

use self::{interaction::handle_interaction, message::handle_message};

use super::Context;

mod interaction;
mod message;

#[derive(Debug)]
enum ProcessResult {
    Success,
    NoSendPermission,
}

fn log_command(ctx: &Context, cmd: &dyn Authored, name: &str) {
    let username = cmd
        .user()
        .map(|u| u.name.as_str())
        .unwrap_or("<unknown user>");

    let location = CommandLocation { ctx, cmd };
    info!("[{location}] {username} invoked `{name}`");
}

struct CommandLocation<'a> {
    ctx: &'a Context,
    cmd: &'a dyn Authored,
}

impl Display for CommandLocation<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let guild = match self.cmd.guild_id() {
            Some(id) => id,
            None => return f.write_str("Private"),
        };

        match self.ctx.cache.guild(guild, |g| write!(f, "{}:", g.name())) {
            Ok(Ok(_)) => {
                let channel_res = self.ctx.cache.channel(self.cmd.channel_id(), |c| {
                    f.write_str(c.name.as_deref().unwrap_or("<uncached channel>"))
                });

                match channel_res {
                    Ok(Ok(_)) => Ok(()),
                    Ok(err) => err,
                    Err(_) => f.write_str("<uncached channel>"),
                }
            }
            Ok(err) => err,
            Err(_) => f.write_str("<uncached guild>"),
        }
    }
}

pub async fn event_loop(ctx: Arc<Context>, mut events: Events) {
    while let Some((shard_id, event)) = events.next().await {
        ctx.cache.update(&event);
        ctx.standby.process(&event);
        let ctx = Arc::clone(&ctx);

        tokio::spawn(async move {
            let handle_fut = handle_event(ctx, event, shard_id);

            if let Err(report) = handle_fut.await.wrap_err("error while handling event") {
                error!("{report:?}");
            }
        });
    }
}

async fn handle_event(ctx: Arc<Context>, event: Event, shard_id: u64) -> BotResult<()> {
    match event {
        Event::BanAdd(_) => {}
        Event::BanRemove(_) => {}
        Event::ChannelCreate(_) => {}
        Event::ChannelDelete(_) => {}
        Event::ChannelPinsUpdate(_) => {}
        Event::ChannelUpdate(_) => {}
        Event::GatewayHeartbeat(_) => {}
        Event::GatewayHeartbeatAck => {}
        Event::GatewayHello(_) => {}
        Event::GatewayInvalidateSession(reconnect) => {
            if reconnect {
                warn!(
                    "Gateway has invalidated session for shard {shard_id}, but its reconnectable"
                );
            } else {
                warn!("Gateway has invalidated session for shard {shard_id}");
            }
        }
        Event::GatewayReconnect => {
            info!("Gateway requested shard {shard_id} to reconnect");
        }
        Event::GiftCodeUpdate => {}
        Event::GuildCreate(_) => {}
        Event::GuildDelete(_) => {}
        Event::GuildEmojisUpdate(_) => {}
        Event::GuildIntegrationsUpdate(_) => {}
        Event::GuildStickersUpdate(_) => {}
        Event::GuildUpdate(_) => {}
        Event::IntegrationCreate(_) => {}
        Event::IntegrationDelete(_) => {}
        Event::IntegrationUpdate(_) => {}
        Event::InteractionCreate(e) => handle_interaction(ctx, e.0).await,
        Event::InviteCreate(_) => {}
        Event::InviteDelete(_) => {}
        Event::MemberAdd(_) => {}
        Event::MemberRemove(_) => {}
        Event::MemberUpdate(_) => {}
        Event::MemberChunk(_) => {}
        Event::MessageCreate(msg) => {
            handle_message(ctx, msg.0).await;
        }
        Event::MessageDelete(_) => {}
        Event::MessageDeleteBulk(_) => {}
        Event::MessageUpdate(_) => {}
        Event::PresenceUpdate(_) => {}
        Event::PresencesReplace => {}
        Event::ReactionAdd(_) => {}
        Event::ReactionRemove(_) => {}
        Event::ReactionRemoveAll(_) => {}
        Event::ReactionRemoveEmoji(_) => {}
        Event::Ready(_) => {
            info!("Shard {shard_id} is ready");
        }
        Event::Resumed => info!("Shard {shard_id} is resumed"),
        Event::RoleCreate(_) => {}
        Event::RoleDelete(_) => {}
        Event::RoleUpdate(_) => {}
        Event::ShardConnected(_) => info!("Shard {shard_id} is connected"),
        Event::ShardConnecting(_) => info!("Shard {shard_id} is connecting..."),
        Event::ShardDisconnected(_) => info!("Shard {shard_id} is disconnected"),
        Event::ShardIdentifying(_) => info!("Shard {shard_id} is identifying..."),
        Event::ShardReconnecting(_) => info!("Shard {shard_id} is reconnecting..."),
        Event::ShardPayload(_) => {}
        Event::ShardResuming(_) => info!("Shard {shard_id} is resuming..."),
        Event::StageInstanceCreate(_) => {}
        Event::StageInstanceDelete(_) => {}
        Event::StageInstanceUpdate(_) => {}
        Event::ThreadCreate(_) => {}
        Event::ThreadDelete(_) => {}
        Event::ThreadListSync(_) => {}
        Event::ThreadMemberUpdate(_) => {}
        Event::ThreadMembersUpdate(_) => {}
        Event::ThreadUpdate(_) => {}
        Event::TypingStart(_) => {}
        Event::UnavailableGuild(_) => {}
        Event::UserUpdate(_) => {}
        Event::VoiceServerUpdate(_) => {}
        Event::VoiceStateUpdate(_) => {}
        Event::WebhooksUpdate(_) => {}
    }

    Ok(())
}
