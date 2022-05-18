use twilight_validate::message::MessageValidationError;

pub use self::{map_file::MapFileError, pp::PpError};

mod map_file;
mod pp;

#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => {
        return Err($crate::Error::Custom(format!("{}", format_args!($($arg)*))))
    };
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("missing value in cache")]
    Cache(#[from] crate::core::CacheMiss),
    #[error("error occured on cluster request")]
    ClusterCommand(#[from] twilight_gateway::cluster::ClusterCommandError),
    #[error("failed to start cluster")]
    ClusterStart(#[from] twilight_gateway::cluster::ClusterStartError),
    #[error("chrono parse error")]
    ChronoParse(#[from] chrono::format::ParseError),
    #[error("custom client error")]
    CustomClient(#[from] crate::custom_client::CustomClientError),
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("fmt error")]
    Fmt(#[from] std::fmt::Error),
    #[error("io error")]
    Io(#[from] tokio::io::Error),
    #[error("error while preparing beatmap file")]
    MapFile(#[from] MapFileError),
    #[error("failed to validate message")]
    MessageValidation(#[from] MessageValidationError),
    #[error("missing env variable `{0}`")]
    MissingEnvVariable(&'static str),
    #[error("event was expected to contain member or user but contained neither")]
    MissingAuthor,
    #[error("osu error")]
    Osu(#[from] rosu_v2::error::OsuError),
    #[error("failed to parse env variable `{name}={value}`; expected {expected}")]
    ParsingEnvVariable {
        name: &'static str,
        value: String,
        expected: &'static str,
    },
    #[error("received invalid options for command")]
    ParseSlashOptions(#[from] twilight_interactions::error::ParseError),
    #[error("error while calculating pp")]
    Pp(#[from] PpError),
    #[error("failed to send reaction after {0} retries")]
    ReactionRatelimit(usize),
    #[error("serde json error")]
    Json(#[from] serde_json::Error),
    #[error("shard command error")]
    ShardCommand(#[from] twilight_gateway::shard::CommandError),
    #[error("twilight failed to deserialize response")]
    TwilightDeserialize(#[from] twilight_http::response::DeserializeBodyError),
    #[error("error while making discord request")]
    TwilightHttp(#[from] twilight_http::Error),
}
