use std::pin::Pin;

use futures::Future;
use radix_trie::{Trie, TrieCommon};
use twilight_model::application::command::Command;

use crate::{
    commands::{osu::*, utility::*},
    BotResult,
};

pub use self::command::SlashCommand;

mod command;

macro_rules! slash_trie {
    ($($cmd:ident => $fun:ident,)*) => {
        use twilight_interactions::command::CreateCommand;

        let mut trie = Trie::new();

        $(trie.insert($cmd::NAME, &$fun);)*

        SlashCommands(trie)
    }
}

lazy_static::lazy_static! {
    pub static ref SLASH_COMMANDS: SlashCommands = {
        slash_trie! {
            Leaderboard => LEADERBOARD_SLASH,
            Nlb => NLB_SLASH,
            Ping => PING_SLASH,
        }
    };
}

pub struct SlashCommands(Trie<&'static str, &'static SlashCommand>);

pub type CommandResult = Pin<Box<dyn Future<Output = BotResult<()>> + 'static + Send>>;

impl SlashCommands {
    pub fn command(&self, command: &str) -> Option<&'static SlashCommand> {
        self.0.get(command).copied()
    }

    pub fn collect(&self) -> Vec<Command> {
        self.0.values().map(|c| (c.create)().into()).collect()
    }
}
