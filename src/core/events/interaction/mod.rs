use std::sync::Arc;

use twilight_model::application::interaction::Interaction;

use crate::core::Context;

use self::command::handle_command;

mod command;

pub async fn handle_interaction(ctx: Arc<Context>, interaction: Interaction) {
    if let Interaction::ApplicationCommand(cmd) = interaction {
        handle_command(ctx, cmd).await
    }
}
