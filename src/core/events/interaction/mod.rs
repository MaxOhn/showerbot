use std::sync::Arc;

use twilight_model::application::interaction::Interaction;

use crate::core::Context;

use self::command::handle_command;

mod command;

pub async fn handle_interaction(ctx: Arc<Context>, interaction: Interaction) {
    match interaction {
        Interaction::ApplicationCommand(cmd) => handle_command(ctx, cmd).await,
        _ => {}
    }
}
