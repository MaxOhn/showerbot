use std::{mem, sync::Arc};

use eyre::Report;
use twilight_model::application::interaction::ApplicationCommand;

use crate::{
    core::{
        commands::slash::{SlashCommand, SLASH_COMMANDS},
        events::{log_command, ProcessResult},
        Context,
    },
    util::ApplicationCommandExt,
    BotResult,
};

pub async fn handle_command(ctx: Arc<Context>, mut command: Box<ApplicationCommand>) {
    let name = mem::take(&mut command.data.name);
    log_command(&ctx, &*command, &name);

    let slash = match SLASH_COMMANDS.command(&name) {
        Some(slash) => slash,
        None => return error!("unknown slash command `{name}`"),
    };

    match process_command(ctx, command, slash).await {
        Ok(ProcessResult::Success) => info!("Processed slash command `{name}`"),
        Ok(res) => info!("Command `/{name}` was not processed: {res:?}"),
        Err(err) => {
            let wrap = format!("failed to process slash command `{name}`");
            error!("{:?}", Report::new(err).wrap_err(wrap));
        }
    }
}

async fn process_command(
    ctx: Arc<Context>,
    command: Box<ApplicationCommand>,
    slash: &SlashCommand,
) -> BotResult<ProcessResult> {
    if slash.flags.defer() {
        command.defer(&ctx, slash.flags.ephemeral()).await?;
    }

    (slash.exec)(ctx, command).await?;

    Ok(ProcessResult::Success)
}
