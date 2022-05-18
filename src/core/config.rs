use std::{env, mem::MaybeUninit, path::PathBuf};

use hashbrown::HashMap;
use once_cell::sync::OnceCell;
use rosu_v2::model::Grade;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, UserMarker},
    Id,
};

use crate::{util::Emote, BotResult, Error};

pub static CONFIG: OnceCell<BotConfig> = OnceCell::new();

#[derive(Debug)]
pub struct BotConfig {
    pub database_url: String,
    pub tokens: Tokens,
    pub paths: Paths,
    grades: [String; 9],
    pub emotes: HashMap<Emote, String>,
    pub owner: Id<UserMarker>,
    pub dev_guild: Id<GuildMarker>,
}

#[derive(Debug)]
pub struct Paths {
    pub maps: PathBuf,
}

#[derive(Debug)]
pub struct Tokens {
    pub discord: String,
    pub osu_client_id: u64,
    pub osu_client_secret: String,
    pub osu_session: String,
}

impl BotConfig {
    pub fn init() -> BotResult<()> {
        let mut grades = [
            MaybeUninit::uninit(),
            MaybeUninit::uninit(),
            MaybeUninit::uninit(),
            MaybeUninit::uninit(),
            MaybeUninit::uninit(),
            MaybeUninit::uninit(),
            MaybeUninit::uninit(),
            MaybeUninit::uninit(),
            MaybeUninit::uninit(),
        ];

        let grade_strs = ["F", "D", "C", "B", "A", "S", "X", "SH", "XH"];

        for grade_str in grade_strs {
            let key: Grade = grade_str.parse().unwrap();
            let value: String = env_var(grade_str)?;
            grades[key as usize].write(value);
        }

        // SAFETY: All grades have been initialized.
        // Otherwise an error would have been thrown due to a missing emote.
        let grades = unsafe { (&grades as *const _ as *const [String; 9]).read() };

        let emotes = [
            "jump_start",
            "single_step_back",
            "single_step",
            "jump_end",
            "miss",
        ];

        let emotes = emotes
            .iter()
            .map(|emote_str| {
                let key = emote_str.parse().unwrap();
                let value = env_var(emote_str)?;

                Ok((key, value))
            })
            .collect::<BotResult<_>>()?;

        let config = BotConfig {
            database_url: env_var("DATABASE_URL")?,
            tokens: Tokens {
                discord: env_var("DISCORD_TOKEN")?,
                osu_client_id: env_var("OSU_CLIENT_ID")?,
                osu_client_secret: env_var("OSU_CLIENT_SECRET")?,
                osu_session: env_var("OSU_SESSION")?,
            },
            paths: Paths {
                maps: env_var("MAP_PATH")?,
            },
            grades,
            emotes,
            owner: env_var("OWNER_USER_ID")?,
            dev_guild: env_var("DEV_GUILD_ID")?,
        };

        if CONFIG.set(config).is_err() {
            warn!("CONFIG was already set");
        }

        Ok(())
    }

    pub fn grade(&self, grade: Grade) -> &str {
        self.grades[grade as usize].as_str()
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
