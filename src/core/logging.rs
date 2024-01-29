use time::macros::format_description;
use tracing_subscriber::{fmt::time::UtcTime, EnvFilter, FmtSubscriber};

pub fn init() {
    let formatter = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::builder().parse("info").unwrap())
        .with_target(false)
        .with_timer(UtcTime::new(formatter))
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("failed to set global subscriber");
}
