use std::{future::IntoFuture, slice};

use twilight_http::response::ResponseFuture;
use twilight_model::{
    channel::Message,
    id::{marker::ChannelMarker, Id},
};

use crate::{
    core::Context,
    util::{
        builder::{EmbedBuilder, MessageBuilder},
        constants::RED,
    },
};

pub trait ChannelExt {
    /// Create a message inside a green embed
    fn create_message(
        &self,
        ctx: &Context,
        builder: &MessageBuilder<'_>,
    ) -> ResponseFuture<Message>;

    /// Create a message inside a red embed
    fn error(&self, ctx: &Context, content: impl Into<String>) -> ResponseFuture<Message>;
}

impl ChannelExt for Id<ChannelMarker> {
    fn create_message(
        &self,
        ctx: &Context,
        builder: &MessageBuilder<'_>,
    ) -> ResponseFuture<Message> {
        let mut req = ctx.http.create_message(*self);

        if let Some(ref content) = builder.content {
            req = req.content(content.as_ref()).expect("invalid content");
        }

        if let Some(ref embed) = builder.embed {
            req = req.embeds(slice::from_ref(embed)).expect("invalid embed");
        }

        if let Some(components) = builder.components.as_deref() {
            req = req.components(components).expect("invalid components");
        }

        let req = match builder.attachment {
            Some(ref attachment) => req.attachments(slice::from_ref(attachment)).unwrap(),
            None => req,
        };

        req.into_future()
    }

    #[inline]
    fn error(&self, ctx: &Context, content: impl Into<String>) -> ResponseFuture<Message> {
        let embed = EmbedBuilder::new().color(RED).description(content).build();

        ctx.http
            .create_message(*self)
            .embeds(&[embed])
            .expect("invalid embed")
            .into_future()
    }
}

impl ChannelExt for Message {
    #[inline]
    fn create_message(
        &self,
        ctx: &Context,
        builder: &MessageBuilder<'_>,
    ) -> ResponseFuture<Message> {
        self.channel_id.create_message(ctx, builder)
    }

    #[inline]
    fn error(&self, ctx: &Context, content: impl Into<String>) -> ResponseFuture<Message> {
        self.channel_id.error(ctx, content)
    }
}
