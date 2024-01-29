use std::{
    fmt::{Display, Error, Formatter},
    sync::Arc,
};

use futures::StreamExt;
use twilight_gateway::{stream::ShardEventStream, Event, Shard};

use crate::{util::Authored, BotResult};

pub use self::interaction::InteractionCommand;

use self::{interaction::handle_interaction, message::handle_message};

use super::Context;

mod interaction;
mod message;

fn log_command(cmd: &dyn Authored, name: &str) {
    let username = cmd
        .user()
        .map(|u| u.name.as_str())
        .unwrap_or("<unknown user>");

    let location = CommandLocation { cmd };
    info!("[{location}] {username} invoked `{name}`");
}

struct CommandLocation<'a> {
    cmd: &'a dyn Authored,
}

impl Display for CommandLocation<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let guild = match self.cmd.guild_id() {
            Some(id) => id,
            None => return f.write_str("Private"),
        };

        write!(f, "{{Guild {guild}}}:{{Channel {}}}", self.cmd.channel_id())
    }
}

pub async fn event_loop(ctx: Arc<Context>, shards: &mut [Shard]) {
    let mut stream = ShardEventStream::new(shards.iter_mut());

    // actual event loop
    'event_loop: loop {
        let err = match stream.next().await {
            Some((shard, Ok(event))) => {
                ctx.standby.process(&event);
                let ctx = Arc::clone(&ctx);
                let shard_id = shard.id().number();

                tokio::spawn(async move {
                    if let Err(err) = handle_event(ctx, event, shard_id).await {
                        error!(?err, "Failed to handle event");
                    }
                });

                continue 'event_loop;
            }
            Some((_, Err(err))) => Some(err),
            None => return,
        };

        if let Some(err) = err {
            error!(%err, "Event error");

            if err.is_fatal() {
                return;
            }
        }
    }
}

async fn handle_event(ctx: Arc<Context>, event: Event, shard_id: u64) -> BotResult<()> {
    match event {
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
        Event::InteractionCreate(e) => handle_interaction(ctx, e.0).await,
        Event::MessageCreate(msg) => handle_message(ctx, msg.0).await,
        Event::Ready(_) => info!("Shard {shard_id} is ready"),
        Event::Resumed => info!("Shard {shard_id} is resumed"),
        _ => {}
    }

    Ok(())
}
