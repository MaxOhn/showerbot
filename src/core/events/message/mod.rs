use std::sync::Arc;

use eyre::Report;
use twilight_model::channel::Message;

use crate::{
    core::{
        commands::prefix::{Args, PrefixCommand, Stream},
        Context,
    },
    BotResult,
};

use self::parse::*;

use super::log_command;

mod parse;

pub async fn handle_message(ctx: Arc<Context>, msg: Message) {
    // Ignore bots and webhooks
    if msg.author.bot || msg.webhook_id.is_some() {
        return;
    }

    // Check msg content for a prefix
    let mut stream = Stream::new(&msg.content);
    stream.take_while_char(char::is_whitespace);

    let prefix = ctx
        .prefixes
        .iter()
        .find(|prefix| stream.starts_with(prefix));

    if let Some(prefix) = prefix {
        stream.increment(prefix.len());
    } else if msg.guild_id.is_some() {
        return;
    }

    // Parse msg content for commands
    let (cmd, num) = match parse_invoke(&mut stream) {
        Invoke::Command { cmd, num } => (cmd, num),
        Invoke::None => return,
    };

    let name = cmd.name();
    log_command(&msg, name);

    match process_command(ctx, cmd, &msg, stream, num).await {
        Ok(_) => info!("Processed command `{name}`"),
        Err(err) => {
            let wrap = format!("failed to process prefix command `{name}`");
            error!("{:?}", Report::new(err).wrap_err(wrap));
        }
    }
}

async fn process_command(
    ctx: Arc<Context>,
    cmd: &PrefixCommand,
    msg: &Message,
    stream: Stream<'_>,
    num: Option<u64>,
) -> BotResult<()> {
    let channel = msg.channel_id;

    // Prepare lightweight arguments
    let args = Args::new(&msg.content, stream, num);

    // Broadcast typing event
    if cmd.flags.defer() {
        let _ = ctx.http.create_typing_trigger(channel).await;
    }

    // Call command function
    (cmd.exec)(ctx, msg, args).await
}
