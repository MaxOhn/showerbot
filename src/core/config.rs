use std::{env, path::PathBuf};

use once_cell::sync::OnceCell;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, UserMarker},
    Id,
};

use crate::{BotResult, Error};

pub static CONFIG: OnceCell<BotConfig> = OnceCell::new();

pub struct BotConfig {
    pub tokens: Tokens,
    pub paths: Paths,
    pub prefixes: Box<[Box<str>]>,
}

pub struct Paths {
    pub maps: PathBuf,
}

pub struct Tokens {
    pub discord: String,
    pub osu_client_id: u64,
    pub osu_client_secret: String,
    pub osu_session: String,
}

impl BotConfig {
    pub fn init() -> BotResult<()> {
        let Prefixes(prefixes) =
            env_var("PREFIX")
                .or_else(|_| env_var("PREFIXES"))
                .map_err(|e| match e {
                    Error::ParsingEnvVariable { name, value, .. } => Error::ParsingEnvVariable {
                        name,
                        value,
                        expected: "string of whitespace-separated prefixes",
                    },
                    e => e,
                })?;

        let config = BotConfig {
            tokens: Tokens {
                discord: env_var("DISCORD_TOKEN")?,
                osu_client_id: env_var("OSU_CLIENT_ID")?,
                osu_client_secret: env_var("OSU_CLIENT_SECRET")?,
                osu_session: env_var("OSU_SESSION")?,
            },
            paths: Paths {
                maps: env_var("MAP_PATH")?,
            },
            prefixes,
        };

        if CONFIG.set(config).is_err() {
            warn!("CONFIG was already set");
        }

        Ok(())
    }
}

trait EnvKind: Sized {
    const EXPECTED: &'static str;

    fn from_str(s: &str) -> Option<Self>;
}

macro_rules! env_kind {
    ($($ty:ty: $arg:ident => $impl:block,)*) => {
        $(
            impl EnvKind for $ty {
                const EXPECTED: &'static str = stringify!($ty);

                fn from_str($arg: &str) -> Option<Self> {
                    $impl
                }
            }
        )*
    };
}

env_kind! {
    u16: s => { s.parse().ok() },
    u64: s => { s.parse().ok() },
    PathBuf: s => { s.parse().ok() },
    String: s => { Some(s.to_owned()) },
    Id<UserMarker>: s => { s.parse().ok().map(Id::new) },
    Id<GuildMarker>: s => { s.parse().ok().map(Id::new) },
    Id<ChannelMarker>: s => { s.parse().ok().map(Id::new) },
    Prefixes: s => {
        let prefixes = s
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(Box::from)
            .collect();

        Some(Prefixes(prefixes))
    },
    [u8; 4]: s => {
        if !(s.starts_with('[') && s.ends_with(']')) {
            return None
        }

        let mut values = s[1..s.len() - 1].split(',');

        let array = [
            values.next()?.trim().parse().ok()?,
            values.next()?.trim().parse().ok()?,
            values.next()?.trim().parse().ok()?,
            values.next()?.trim().parse().ok()?,
        ];

        Some(array)
    },
}

fn env_var<T: EnvKind>(name: &'static str) -> BotResult<T> {
    let value = env::var(name).map_err(|_| Error::MissingEnvVariable(name))?;

    T::from_str(&value).ok_or(Error::ParsingEnvVariable {
        name,
        value,
        expected: T::EXPECTED,
    })
}

struct Prefixes(Box<[Box<str>]>);
