use crate::{
    core::commands::prefix::{PrefixCommand, Stream, PREFIX_COMMANDS},
    util::CowUtils,
};

pub enum Invoke {
    Command { cmd: &'static PrefixCommand },
    None,
}

pub fn parse_invoke(stream: &mut Stream<'_>) -> Invoke {
    let name = stream
        .take_until_char(char::is_whitespace)
        .cow_to_ascii_lowercase();

    stream.take_while_char(char::is_whitespace);

    if let Some(cmd) = PREFIX_COMMANDS.command(name.as_ref()) {
        Invoke::Command { cmd }
    } else {
        Invoke::None
    }
}
