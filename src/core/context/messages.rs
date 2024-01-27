use twilight_model::{
    channel::Message,
    id::{marker::ChannelMarker, Id},
};

use crate::{BotResult, Context, Error};

impl Context {
    pub async fn retrieve_channel_history(
        &self,
        channel_id: Id<ChannelMarker>,
    ) -> BotResult<Vec<Message>> {
        self.http
            .channel_messages(channel_id)
            .limit(50)
            .unwrap()
            .exec()
            .await?
            .models()
            .await
            .map_err(Error::from)
    }
}
