use smallstr::SmallString;
use smallvec::SmallVec;

pub type Prefix = SmallString<[u8; 2]>;
pub type Prefixes = SmallVec<[Prefix; 5]>;

#[derive(Debug, Clone)]
pub struct GuildConfig {
    pub prefixes: Prefixes,
}

impl Default for GuildConfig {
    fn default() -> Self {
        GuildConfig {
            prefixes: smallvec::smallvec!["<".into()],
        }
    }
}
