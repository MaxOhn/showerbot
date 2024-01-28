use std::{collections::BTreeMap, fmt::Write, sync::Arc, time::Duration};

use command_macros::command;
use eyre::Report;
use hashbrown::HashSet;
use tokio::time::{interval, MissedTickBehavior};
use twilight_model::channel::{message::embed::EmbedField, Message};

use crate::{
    core::{
        commands::prefix::{PrefixCommand, PREFIX_COMMANDS},
        Context,
    },
    util::{
        builder::{EmbedBuilder, FooterBuilder, MessageBuilder},
        constants::{BATHBOT_GITHUB, DESCRIPTION_SIZE},
        levenshtein_distance, ChannelExt,
    },
    BotResult,
};

use super::failed_message_content;

#[command]
#[desc("Display help for prefix commands")]
#[group(Utility)]
#[alias("h")]
#[usage("[command]")]
#[example("", "nlb", "ping")]
async fn prefix_help(ctx: Arc<Context>, msg: &Message, mut args: Args<'_>) -> BotResult<()> {
    match args.next() {
        Some(arg) => match PREFIX_COMMANDS.command(arg) {
            Some(cmd) => command_help(ctx, msg, cmd).await,
            None => failed_help(ctx, msg, arg).await,
        },
        None => dm_help(ctx, msg).await,
    }
}

async fn failed_help(ctx: Arc<Context>, msg: &Message, name: &str) -> BotResult<()> {
    let mut seen = HashSet::new();

    let dists: BTreeMap<_, _> = PREFIX_COMMANDS
        .iter()
        .filter(|cmd| seen.insert(cmd.name()))
        .flat_map(|cmd| cmd.names.iter())
        .map(|&cmd| (levenshtein_distance(name, cmd).0, cmd))
        .filter(|(dist, _)| *dist < 4)
        .collect();

    let content = failed_message_content(dists);
    msg.error(&ctx, content).await?;

    Ok(())
}

async fn command_help(ctx: Arc<Context>, msg: &Message, cmd: &PrefixCommand) -> BotResult<()> {
    let name = cmd.name();
    let prefix = ctx.prefixes.first().map_or("", |prefix| prefix.as_ref());
    let mut fields = Vec::new();

    let eb = EmbedBuilder::new()
        .title(name)
        .description(cmd.help.unwrap_or(cmd.desc));

    let mut usage_len = 0;

    if let Some(usage) = cmd.usage {
        let value = format!("`{prefix}{name} {usage}`");
        usage_len = value.chars().count();

        let field = EmbedField {
            name: "How to use".to_owned(),
            value,
            inline: usage_len <= 29,
        };

        fields.push(field);
    }

    let mut examples = cmd.examples.iter();

    if let Some(first) = examples.next() {
        let len: usize = cmd.examples.iter().map(|&e| name.len() + e.len() + 4).sum();
        let mut value = String::with_capacity(len);
        let mut example_len = 0;
        let cmd_len = prefix.chars().count() + name.chars().count();
        writeln!(value, "`{prefix}{name} {first}`")?;

        for example in examples {
            writeln!(value, "`{prefix}{name} {example}`")?;
            example_len = example_len.max(cmd_len + example.chars().count());
        }

        let not_inline = (usage_len <= 29 && cmd.names.len() > 1 && example_len > 27)
            || ((usage_len > 29 || cmd.names.len() > 1) && example_len > 36)
            || example_len > 45;

        let field = EmbedField {
            name: "Examples".to_owned(),
            value,
            inline: !not_inline,
        };

        fields.push(field);
    }

    let mut aliases = cmd.names.iter().skip(1);

    if let Some(first) = aliases.next() {
        let len: usize = cmd.names.iter().skip(1).map(|n| 4 + n.len()).sum();
        let mut value = String::with_capacity(len);
        write!(value, "`{first}`")?;

        for &alias in aliases {
            write!(value, ", `{alias}`")?;
        }

        let field = EmbedField {
            name: "Aliases".to_owned(),
            value,
            inline: true,
        };

        fields.push(field);
    }

    let footer_text = "Available in servers and DMs";
    let footer = FooterBuilder::new(footer_text);

    let embed = eb.footer(footer).fields(fields).build();
    let builder = MessageBuilder::new().embed(embed);

    msg.create_message(&ctx, &builder).await?;

    Ok(())
}

fn description(ctx: &Context) -> String {
    format!(
        "Prefixes: {:?} (or none in DMs).\n\
        This bot is based on [Bathbot]({BATHBOT_GITHUB}).\n\
        Its main functionality is the national map leaderboard command.\n\
        To find out more about a command like what arguments you can give or which shorter aliases it has, \
        use __**`<help [command]`**__, e.g. `<help nlb`.\n\
        \n__**All commands:**__\n", ctx.prefixes
    )
}

macro_rules! send_chunk {
    ($ctx:ident, $msg:ident, $content:ident, $interval:ident) => {
        let embed = EmbedBuilder::new().description($content).build();
        let builder = MessageBuilder::new().embed(embed);
        $interval.tick().await;

        if let Err(err) = $msg.create_message(&$ctx, &builder).await {
            let report = Report::new(err).wrap_err("error while sending help chunk");
            warn!("{report:?}");
            let content = "Could not DM you, perhaps you disabled it?";
            $msg.error(&$ctx, content).await?;

            return Ok(());
        }
    };
}

async fn dm_help(ctx: Arc<Context>, msg: &Message) -> BotResult<()> {
    let owner = msg.author.id;
    let channel_result = ctx.http.create_private_channel(owner).await;

    let channel = match channel_result {
        Ok(channel_res) => channel_res.model().await?.id,
        Err(err) => {
            let content = "Your DMs seem blocked :(\n\
            Perhaps you disabled incoming messages from other server members?";
            let report = Report::new(err).wrap_err("error while creating DM channel");
            warn!("{report:?}");
            msg.error(&ctx, content).await?;

            return Ok(());
        }
    };

    if msg.guild_id.is_some() {
        let content = "Don't mind me sliding into your DMs :eyes:";
        let builder = MessageBuilder::new().embed(content);
        let _ = msg.create_message(&ctx, &builder).await;
    }

    let mut buf = description(&ctx);
    let mut size = buf.len();
    let mut next_size;

    debug_assert!(
        size < DESCRIPTION_SIZE,
        "description size {size} > {DESCRIPTION_SIZE}",
    );

    let mut cmds: Vec<_> = PREFIX_COMMANDS.iter().collect();

    cmds.sort_unstable_by(|a, b| a.group.cmp(&b.group).then_with(|| a.name().cmp(b.name())));
    cmds.dedup_by_key(|cmd| cmd.name());

    let mut interval = interval(Duration::from_millis(100));
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    for cmd in cmds {
        let name = cmd.name();

        next_size = 5 + name.len() + cmd.desc.len();

        if size + next_size > DESCRIPTION_SIZE {
            send_chunk!(ctx, channel, buf, interval);
            buf = String::with_capacity(DESCRIPTION_SIZE);
            size = 0;
        }

        size += next_size;
        let _ = writeln!(buf, "`{name}`: {}", cmd.desc,);
    }

    if !buf.is_empty() {
        send_chunk!(ctx, channel, buf, interval);
    }

    Ok(())
}
