use std::pin::Pin;

use eyre::{Result, WrapErr};
use futures::Future;
use radix_trie::{Trie, TrieCommon};
use twilight_http::client::InteractionClient;
use twilight_interactions::command::{ApplicationCommandData, CommandOptionExt};

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

    pub async fn register(&self, client: &InteractionClient<'_>) -> Result<()> {
        info!("Creating {} interaction commands...", self.0.len());

        for cmd in self.0.values() {
            let cmd = (cmd.create)();
            let name = cmd.name.clone();

            Self::register_slash_command(cmd, client)
                .await
                .wrap_err_with(|| format!("Failed to register slash command `{name}`"))?
        }

        Ok(())
    }

    async fn register_slash_command(
        cmd: ApplicationCommandData,
        client: &InteractionClient<'_>,
    ) -> Result<()> {
        let options: Vec<_> = cmd
            .options
            .into_iter()
            .map(CommandOptionExt::into)
            .collect();

        client
            .create_global_command()
            .chat_input(&cmd.name, &cmd.description)?
            .command_options(&options)?
            .exec()
            .await
            .wrap_err("Failed to create command")?;

        Ok(())
    }
}
