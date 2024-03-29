mod osu;

use twilight_model::channel::message::embed::Embed;

pub use self::osu::*;

pub trait EmbedData {
    fn build(self) -> Embed;
}

impl EmbedData for Embed {
    fn build(self) -> Embed {
        self
    }
}
