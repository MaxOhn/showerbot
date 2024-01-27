use std::sync::Arc;

use eyre::Report;
use twilight_model::{channel::Message, guild::Permissions};

use crate::{
    core::{
        commands::prefix::{Args, PrefixCommand, Stream},
        Context,
    },
    BotResult,
};

use self::parse::*;

use super::{log_command, ProcessResult};

mod parse;

pub async fn handle_message(ctx: Arc<Context>, msg: Message) {
    // Ignore bots and webhooks
    if msg.author.bot || msg.webhook_id.is_some() {
        return;
    }

    // Check msg content for a prefix
    let mut stream = Stream::new(&msg.content);
    stream.take_while_char(char::is_whitespace);

    // TODO: does msg contain ping to the bot
    let prefix = None::<&str>;

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
    log_command(&ctx, &msg, name);

    match process_command(ctx, cmd, &msg, stream, num).await {
        Ok(ProcessResult::Success) => info!("Processed command `{name}`"),
        Ok(result) => info!("Command `{name}` was not processed: {result:?}"),
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
) -> BotResult<ProcessResult> {
    let channel = msg.channel_id;

    // Does bot have sufficient permissions to send response in a guild?
    if let Some(guild) = msg.guild_id {
        let user = ctx.cache.current_user(|user| user.id)?;
        let permissions = ctx.cache.get_channel_permissions(user, channel, guild);

        if !permissions.contains(Permissions::SEND_MESSAGES) {
            return Ok(ProcessResult::NoSendPermission);
        }
    }

    // Prepare lightweight arguments
    let args = Args::new(&msg.content, stream, num);

    // Broadcast typing event
    if cmd.flags.defer() {
        let _ = ctx.http.create_typing_trigger(channel).exec().await;
    }

    // Call command function
    (cmd.exec)(ctx, msg, args).await?;

    Ok(ProcessResult::Success)
}
