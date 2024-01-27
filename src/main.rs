#![warn(clippy::all, nonstandard_style, rust_2018_idioms, unused, warnings)]

#[macro_use]
extern crate async_trait;

#[macro_use]
extern crate tracing;

#[macro_use]
mod error;

mod commands;
mod core;
mod custom_client;
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
    let _log_worker_guard = logging::initialize();

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
    dotenvy::dotenv().map_err(|_| {
        eyre::eyre!(
            "Failed to load env variables. \
            Be sure you copied the .env.example file from the repository in \
            the same directory as this executable, renamed it to .env, and \
            adjusted its content."
        )
    })?;

    // Load config file
    core::BotConfig::init().context("failed to initialize config")?;

    let (ctx, events) = Context::new().await.context("failed to create ctx")?;

    let ctx = Arc::new(ctx);

    // Initialize commands
    PREFIX_COMMANDS.init();

    SLASH_COMMANDS
        .register(&ctx.interaction())
        .await
        .wrap_err("failed to register slash commands")?;

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
