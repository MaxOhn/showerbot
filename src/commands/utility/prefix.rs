use command_macros::command;

use crate::{
    database::Prefix,
    util::{builder::MessageBuilder, constants::GENERAL_ISSUE, matcher, ChannelExt},
    BotResult, Context,
};

use std::{cmp::Ordering, fmt::Write, sync::Arc};

#[command]
#[desc("Change the prefixes for a server")]
#[help(
    "Change the prefixes for a server.\n\
    To check the current prefixes for this server, \
    don't pass any arguments.\n\
    Otherwise, the first argument must be either `add` or `remove`.\n\
    Following that must be a space-separated list of \
    characters or strings you want to add or remove as prefix.\n\
    Servers must have between one and five prefixes."
)]
#[usage("[add / remove] [prefix]")]
#[example("add $ 🍆 new_pref", "remove < !!")]
#[alias("prefixes")]
#[flags(ONLY_GUILDS, ONLY_OWNER, SKIP_DEFER)]
#[group(Utility)]
async fn prefix_prefix(ctx: Arc<Context>, msg: &Message, mut args: Args<'_>) -> BotResult<()> {
    let guild_id = msg.guild_id.unwrap();

    let action = match args.next() {
        Some("add") | Some("a") => Action::Add,
        Some("remove") | Some("r") => Action::Remove,
        Some(other) => {
            let content = format!(
                "If any arguments are provided, the first one \
                must be either `add` or `remove`, not `{other}`"
            );

            msg.error(&ctx, content).await?;

            return Ok(());
        }
        None => {
            let prefixes = ctx.guild_prefixes(guild_id).await;
            let mut content = String::new();
            current_prefixes(&mut content, &prefixes);
            let builder = MessageBuilder::new().embed(content);
            msg.create_message(&ctx, &builder).await?;

            return Ok(());
        }
    };

    if args.is_empty() {
        let content = "After the first argument you should specify some prefix(es)";
        msg.error(&ctx, content).await?;

        return Ok(());
    }

    let args: Vec<Prefix> = args.take(5).map(|arg| arg.into()).collect();

    if args.iter().any(|arg| matcher::is_custom_emote(arg)) {
        let content = "Does not work with custom emotes unfortunately \\:(";
        msg.error(&ctx, content).await?;

        return Ok(());
    }

    let update_fut = ctx.update_guild_config(guild_id, |config| match action {
        Action::Add => {
            config.prefixes.extend(args);

            config.prefixes.sort_unstable_by(|a, b| {
                if a == "<" {
                    Ordering::Less
                } else if b == "<" {
                    Ordering::Greater
                } else {
                    a.cmp(b)
                }
            });

            config.prefixes.dedup();
            config.prefixes.truncate(5);
        }
        Action::Remove => {
            for arg in args {
                config.prefixes.retain(|p| p != &arg);

                if config.prefixes.is_empty() {
                    config.prefixes.push(arg);

                    break;
                }
            }
        }
    });

    if let Err(err) = update_fut.await {
        let _ = msg.error(&ctx, GENERAL_ISSUE).await;

        return Err(err);
    }

    let mut content = "Prefixes updated!\n".to_owned();
    let prefixes = ctx.guild_prefixes(guild_id).await;
    current_prefixes(&mut content, &prefixes);
    let builder = MessageBuilder::new().embed(content);
    msg.create_message(&ctx, &builder).await?;

    Ok(())
}

enum Action {
    Add,
    Remove,
}

fn current_prefixes(content: &mut String, prefixes: &[Prefix]) {
    content.push_str("Prefixes for this server: ");
    let len = prefixes.iter().map(|p| p.len() + 4).sum();
    content.reserve_exact(len);
    let mut prefixes = prefixes.iter();

    if let Some(first) = prefixes.next() {
        let _ = write!(content, "`{first}`");

        for prefix in prefixes {
            let _ = write!(content, ", `{prefix}`");
        }
    }
}
