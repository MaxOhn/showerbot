use std::{borrow::Cow, future::IntoFuture, mem, slice};

use twilight_http::response::{marker::EmptyBody, ResponseFuture};
use twilight_interactions::command::CommandInputData;
use twilight_model::{
    channel::Message,
    guild::Permissions,
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
};

use crate::{
    core::{Context, InteractionCommand},
    util::{
        builder::{EmbedBuilder, MessageBuilder},
        constants::RED,
    },
};

pub trait InteractionCommandExt {
    /// Extract input data containing options and resolved values
    fn input_data(&mut self) -> CommandInputData<'static>;

    /// Ackowledge the command and respond immediatly.
    fn callback(&self, ctx: &Context, builder: MessageBuilder<'_>) -> ResponseFuture<EmptyBody>;

    /// Ackownledge the command but don't respond yet.
    ///
    /// Must use [`ApplicationCommandExt::update`] afterwards!
    fn defer(&self, ctx: &Context) -> ResponseFuture<EmptyBody>;

    /// After having already ackowledged the command either via
    /// [`ApplicationCommandExt::callback`] or [`ApplicationCommandExt::defer`],
    /// use this to update the response.
    fn update(&self, ctx: &Context, builder: &MessageBuilder<'_>) -> ResponseFuture<Message>;

    /// Update a command to some content in a red embed.
    ///
    /// Be sure the command was deferred beforehand.
    fn error(&self, ctx: &Context, content: impl Into<String>) -> ResponseFuture<Message>;

    /// Respond to a command with some content in a red embed.
    ///
    /// Be sure the command was **not** deferred beforehand.
    fn error_callback(
        &self,
        ctx: &Context,
        content: impl Into<String>,
    ) -> ResponseFuture<EmptyBody>;
}

impl InteractionCommandExt for InteractionCommand {
    fn input_data(&mut self) -> CommandInputData<'static> {
        CommandInputData {
            options: mem::take(&mut self.data.options),
            resolved: self.data.resolved.take().map(Cow::Owned),
        }
    }

    fn callback(&self, ctx: &Context, builder: MessageBuilder<'_>) -> ResponseFuture<EmptyBody> {
        let attachments = builder
            .attachment
            .filter(|_| {
                self.permissions.map_or(true, |permissions| {
                    permissions.contains(Permissions::ATTACH_FILES)
                })
            })
            .map(|attachment| vec![attachment]);

        let data = InteractionResponseData {
            components: builder.components,
            content: builder.content.map(|c| c.into_owned()),
            embeds: builder.embed.map(|e| vec![e]),
            flags: None,
            attachments,
            ..Default::default()
        };

        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };

        ctx.interaction()
            .create_response(self.id, &self.token, &response)
            .into_future()
    }

    fn defer(&self, ctx: &Context) -> ResponseFuture<EmptyBody> {
        let response = InteractionResponse {
            kind: InteractionResponseType::DeferredChannelMessageWithSource,
            data: Some(InteractionResponseData::default()),
        };

        ctx.interaction()
            .create_response(self.id, &self.token, &response)
            .into_future()
    }

    fn update<'l>(
        &'l self,
        ctx: &'l Context,
        builder: &'l MessageBuilder<'l>,
    ) -> ResponseFuture<Message> {
        let client = ctx.interaction();

        let mut req = client.update_response(&self.token);

        if let Some(ref content) = builder.content {
            req = req
                .content(Some(content.as_ref()))
                .expect("invalid content");
        }

        if let Some(ref embed) = builder.embed {
            req = req
                .embeds(Some(slice::from_ref(embed)))
                .expect("invalid embed");
        }

        if let Some(ref components) = builder.components {
            req = req
                .components(Some(components))
                .expect("invalid components");
        }

        if let Some(attachment) = builder.attachment.as_ref().filter(|_| {
            self.permissions.map_or(true, |permissions| {
                permissions.contains(Permissions::ATTACH_FILES)
            })
        }) {
            req = req.attachments(slice::from_ref(attachment)).unwrap();
        }

        req.into_future()
    }

    fn error(&self, ctx: &Context, content: impl Into<String>) -> ResponseFuture<Message> {
        let embed = EmbedBuilder::new().description(content).color(RED).build();

        ctx.interaction()
            .update_response(&self.token)
            .embeds(Some(&[embed]))
            .expect("invalid embed")
            .into_future()
    }

    fn error_callback(
        &self,
        ctx: &Context,
        content: impl Into<String>,
    ) -> ResponseFuture<EmptyBody> {
        let embed = EmbedBuilder::new().description(content).color(RED).build();

        let data = InteractionResponseData {
            embeds: Some(vec![embed]),
            ..Default::default()
        };

        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data),
        };

        ctx.interaction()
            .create_response(self.id, &self.token, &response)
            .into_future()
    }
}
