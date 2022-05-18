use std::pin::Pin;

use futures::Future;
use radix_trie::{Trie, TrieCommon};

use crate::{
    commands::{help::*, osu::*, utility::*},
    BotResult,
};

pub use self::{args::Args, command::PrefixCommand, stream::Stream};

mod args;
mod command;
mod stream;

macro_rules! prefix_trie {
    ($($cmd:ident,)*) => {
        let mut trie = Trie::new();

        $(
            for &name in $cmd.names {
                if trie.insert(name, &$cmd).is_some() {
                    panic!("duplicate prefix command `{name}`");
                }
            }
        )*

        PrefixCommands(trie)
    }
}

lazy_static::lazy_static! {
    pub static ref PREFIX_COMMANDS: PrefixCommands = {
        prefix_trie! {
            HELP_PREFIX,
            NATIONALLEADERBOARD_PREFIX,
            PING_PREFIX,
            PREFIX_PREFIX,
        }
    };
}

pub type CommandResult<'fut> = Pin<Box<dyn Future<Output = BotResult<()>> + 'fut + Send>>;

type PrefixTrie = Trie<&'static str, &'static PrefixCommand>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrefixCommandGroup {
    AllModes,
    Utility,
}

pub struct PrefixCommands(PrefixTrie);

impl PrefixCommands {
    /// Access prefix commands so their lazy_static executes
    pub fn init(&self) {}

    pub fn command(&self, command: &str) -> Option<&'static PrefixCommand> {
        self.0.get(command).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = &'static PrefixCommand> + '_ {
        self.0.values().copied()
    }
}
