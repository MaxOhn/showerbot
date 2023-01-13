#![deny(clippy::all, nonstandard_style, rust_2018_idioms, unused, warnings)]

#[macro_use]
extern crate async_trait;

#[macro_use]
extern crate tracing;

#[macro_use]
mod error;

mod commands;
mod core;
mod custom_client;
mod database;
mod embeds;
mod pagination;
mod pp;
mod util;

use std::sync::Arc;

use eyre::{Result, WrapErr};
use tokio::{runtime::Builder as RuntimeBuilder, signal};

use crate::{
    core::{
        commands::{prefix::PREFIX_COMMANDS, slash::SLASH_COMMANDS},
        event_loop, logging, Context, CONFIG,
    },
    error::Error,
};

type BotResult<T> = Result<T, Error>;

fn main() {
    let runtime = RuntimeBuilder::new_multi_thread()
        .enable_all()
        .thread_stack_size(2 * 1024 * 1024)
        .build()
        .expect("Could not build runtime");

    if let Err(report) = runtime.block_on(async_main()) {
        error!("{:?}", report.wrap_err("critical error in main"));
    }
}

async fn async_main() -> eyre::Result<()> {
    dotenvy::dotenv()?;
    let _log_worker_guard = logging::initialize();

    // Load config file
    core::BotConfig::init().context("failed to initialize config")?;

    let (ctx, events) = Context::new().await.context("failed to create ctx")?;

    let ctx = Arc::new(ctx);

    // Initialize commands
    PREFIX_COMMANDS.init();
    let slash_commands = SLASH_COMMANDS.collect();
    info!("Setting {} slash commands...", slash_commands.len());

    // info!("Defining: {slash_commands:#?}");

    if cfg!(debug_assertions) {
        ctx.interaction()
            .set_global_commands(&[])
            .exec()
            .await
            .context("failed to set empty global commands")?;

        let _received = ctx
            .interaction()
            .set_guild_commands(CONFIG.get().unwrap().dev_guild, &slash_commands)
            .exec()
            .await
            .context("failed to set guild commands")?;

        // let commands = _received.models().await?;
        // info!("Received: {commands:#?}");
    } else {
        ctx.interaction()
            .set_global_commands(&slash_commands)
            .exec()
            .await
            .context("failed to set global commands")?;
    }

    let event_ctx = Arc::clone(&ctx);
    ctx.cluster.up().await;

    tokio::select! {
        _ = event_loop(event_ctx, events) => error!("Event loop ended"),
        res = signal::ctrl_c() => if let Err(report) = res.wrap_err("error while awaiting ctrl+c") {
            error!("{report:?}");
        } else {
            info!("Received Ctrl+C");
        },
    }

    ctx.cluster.down();

    info!("Shutting down");

    Ok(())
}
