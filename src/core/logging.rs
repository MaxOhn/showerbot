use time::macros::format_description;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::time::UtcTime, EnvFilter, FmtSubscriber};

pub fn init() {
    let formatter = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_target(false)
        .with_timer(UtcTime::new(formatter))
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("failed to set global subscriber");
}
