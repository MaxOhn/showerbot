use std::{mem, sync::Arc};

use eyre::Report;

use crate::{
    core::{
        commands::slash::{SlashCommand, SLASH_COMMANDS},
        events::log_command,
        Context,
    },
    util::InteractionCommandExt,
    BotResult,
};

use super::InteractionCommand;

pub async fn handle_command(ctx: Arc<Context>, mut command: InteractionCommand) {
    let name = mem::take(&mut command.data.name);
    log_command(&command, &name);

    let slash = match SLASH_COMMANDS.command(&name) {
        Some(slash) => slash,
        None => return error!("unknown slash command `{name}`"),
    };

    match process_command(ctx, command, slash).await {
        Ok(()) => info!("Processed slash command `{name}`"),
        Err(err) => {
            let wrap = format!("failed to process slash command `{name}`");
            error!("{:?}", Report::new(err).wrap_err(wrap));
        }
    }
}

async fn process_command(
    ctx: Arc<Context>,
    command: InteractionCommand,
    slash: &SlashCommand,
) -> BotResult<()> {
    if slash.flags.defer() {
        command.defer(&ctx).await?;
    }

    (slash.exec)(ctx, command).await
}
