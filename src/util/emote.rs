use std::str::FromStr;

use rosu_v2::prelude::GameMode;
use twilight_http::request::channel::reaction::RequestReactionType;
use twilight_model::id::Id;

use crate::CONFIG;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Emote {
    Std,
    Tko,
    Ctb,
    Mna,

    Osu,
    Twitch,
    Tracking,

    Minimize,
    Expand,

    JumpStart,
    MultiStepBack,
    SingleStepBack,
    MyPosition,
    SingleStep,
    MultiStep,
    JumpEnd,

    Miss,
}

impl Emote {
    pub fn text(self) -> &'static str {
        CONFIG.get().unwrap().emotes.get(&self).unwrap().as_str()
    }

    pub fn request_reaction_type(&self) -> RequestReactionType<'_> {
        let emote = CONFIG.get().unwrap().emotes.get(self);

        let (id, name) = emote
            .unwrap_or_else(|| panic!("No {self:?} emote in config"))
            .split_emote();

        RequestReactionType::Custom {
            id: Id::new(id),
            name: Some(name),
        }
    }
}

impl From<GameMode> for Emote {
    fn from(mode: GameMode) -> Self {
        match mode {
            GameMode::Osu => Self::Std,
            GameMode::Taiko => Self::Tko,
            GameMode::Catch => Self::Ctb,
            GameMode::Mania => Self::Mna,
        }
    }
}

trait SplitEmote {
    fn split_emote(&self) -> (u64, &str);
}

impl SplitEmote for String {
    fn split_emote(&self) -> (u64, &str) {
        let mut split = self.split(':');
        let name = split.nth(1).unwrap();
        let id = split.next().unwrap();
        let id = u64::from_str(&id[0..id.len() - 1]).unwrap();

        (id, name)
    }
}

impl FromStr for Emote {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let emote = match s {
            "osu" => Self::Osu,
            "osu_std" => Self::Std,
            "osu_taiko" => Self::Tko,
            "osu_ctb" => Self::Ctb,
            "osu_mania" => Self::Mna,
            "twitch" => Self::Twitch,
            "tracking" => Self::Tracking,
            "minimize" => Self::Minimize,
            "expand" => Self::Expand,
            "jump_start" => Self::JumpStart,
            "multi_step_back" => Self::MultiStepBack,
            "single_step_back" => Self::SingleStepBack,
            "my_position" => Self::MyPosition,
            "single_step" => Self::SingleStep,
            "multi_step" => Self::MultiStep,
            "jump_end" => Self::JumpEnd,
            "miss" => Self::Miss,
            _ => return Err(()),
        };

        Ok(emote)
    }
}
