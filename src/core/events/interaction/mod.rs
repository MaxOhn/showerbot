use std::sync::Arc;

use twilight_model::{
    application::interaction::{application_command::CommandData, Interaction, InteractionData},
    guild::{PartialMember, Permissions},
    id::{
        marker::{ChannelMarker, GuildMarker, InteractionMarker},
        Id,
    },
    user::User,
};

use crate::core::Context;

use self::command::handle_command;

mod command;

pub async fn handle_interaction(ctx: Arc<Context>, interaction: Interaction) {
    let Some(cmd) = InteractionCommand::try_new(interaction) else {
        return error!("invalid interaction data");
    };

    handle_command(ctx, cmd).await
}

pub struct InteractionCommand {
    pub channel_id: Id<ChannelMarker>,
    pub data: Box<CommandData>,
    pub guild_id: Option<Id<GuildMarker>>,
    pub id: Id<InteractionMarker>,
    pub member: Option<PartialMember>,
    pub permissions: Option<Permissions>,
    pub token: String,
    pub user: Option<User>,
}

impl InteractionCommand {
    fn try_new(interaction: Interaction) -> Option<Self> {
        let Some(InteractionData::ApplicationCommand(data)) = interaction.data else {
            return None;
        };

        Some(Self {
            channel_id: interaction.channel?.id,
            data,
            guild_id: interaction.guild_id,
            id: interaction.id,
            member: interaction.member,
            permissions: interaction.app_permissions,
            token: interaction.token,
            user: interaction.user,
        })
    }
}
