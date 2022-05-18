use std::borrow::Cow;

use twilight_model::{
    application::component::Component, channel::embed::Embed, http::attachment::Attachment,
};

use super::EmbedBuilder;

#[derive(Default)]
pub struct MessageBuilder<'c> {
    pub content: Option<Cow<'c, str>>,
    pub embed: Option<Embed>,
    pub attachment: Option<Attachment>,
    pub components: Option<Vec<Component>>,
}

impl<'c> MessageBuilder<'c> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn content(mut self, content: impl Into<Cow<'c, str>>) -> Self {
        self.content = Some(content.into());

        self
    }

    pub fn embed(mut self, embed: impl IntoEmbed) -> Self {
        self.embed = Some(embed.into_embed());

        self
    }
}

impl<'c> From<Embed> for MessageBuilder<'c> {
    #[inline]
    fn from(embed: Embed) -> Self {
        Self {
            embed: Some(embed),
            ..Default::default()
        }
    }
}

pub trait IntoEmbed {
    fn into_embed(self) -> Embed;
}

impl IntoEmbed for Embed {
    #[inline]
    fn into_embed(self) -> Embed {
        self
    }
}

impl IntoEmbed for EmbedBuilder {
    #[inline]
    fn into_embed(self) -> Embed {
        self.build()
    }
}

impl IntoEmbed for String {
    #[inline]
    fn into_embed(self) -> Embed {
        EmbedBuilder::new().description(self).build()
    }
}

impl<'s> IntoEmbed for &'s str {
    #[inline]
    fn into_embed(self) -> Embed {
        EmbedBuilder::new().description(self).build()
    }
}
