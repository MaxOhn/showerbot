use twilight_model::{
    channel::Message,
    id::{
        marker::{ChannelMarker, GuildMarker, UserMarker},
        Id,
    },
    user::User,
};

use crate::{core::InteractionCommand, error::Error, BotResult};

pub trait Authored {
    /// Channel id of the event
    fn channel_id(&self) -> Id<ChannelMarker>;

    /// Guild id of the event
    fn guild_id(&self) -> Option<Id<GuildMarker>>;

    /// Author of the event
    fn user(&self) -> BotResult<&User>;

    /// Author's user id
    fn user_id(&self) -> BotResult<Id<UserMarker>>;
}

impl Authored for InteractionCommand {
    fn channel_id(&self) -> Id<ChannelMarker> {
        self.channel_id
    }

    fn guild_id(&self) -> Option<Id<GuildMarker>> {
        self.guild_id
    }

    fn user(&self) -> BotResult<&User> {
        self.member
            .as_ref()
            .and_then(|member| member.user.as_ref())
            .or(self.user.as_ref())
            .ok_or(Error::MissingAuthor)
    }

    fn user_id(&self) -> BotResult<Id<UserMarker>> {
        self.user().map(|user| user.id)
    }
}

impl Authored for Message {
    #[inline]
    fn channel_id(&self) -> Id<ChannelMarker> {
        self.channel_id
    }

    #[inline]
    fn guild_id(&self) -> Option<Id<GuildMarker>> {
        self.guild_id
    }

    #[inline]
    fn user(&self) -> BotResult<&User> {
        Ok(&self.author)
    }

    #[inline]
    fn user_id(&self) -> BotResult<Id<UserMarker>> {
        Ok(self.author.id)
    }
}
