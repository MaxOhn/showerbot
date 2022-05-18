use std::str::FromStr;

use regex::Regex;
use rosu_v2::prelude::{GameMods, UserId as OsuUserId};

use super::{constants::OSU_BASE, osu::ModSelection};

pub fn is_custom_emote(msg: &str) -> bool {
    EMOJI_MATCHER.is_match(msg)
}

#[allow(dead_code)]
pub fn get_osu_user_id(msg: &str) -> Option<OsuUserId> {
    OSU_URL_USER_MATCHER.captures(msg).and_then(|c| {
        c.get(1)
            .and_then(|m| m.as_str().parse().ok())
            .map(OsuUserId::Id)
            .or_else(|| c.get(2).map(|m| OsuUserId::Name(m.as_str().into())))
    })
}

pub fn get_osu_map_id(msg: &str) -> Option<u32> {
    if let Ok(id) = msg.parse() {
        return Some(id);
    }

    if !msg.contains(OSU_BASE) {
        return None;
    }

    let matcher = if let Some(c) = OSU_URL_MAP_OLD_MATCHER.captures(msg) {
        c.get(1)
    } else {
        OSU_URL_MAP_NEW_MATCHER.captures(msg).and_then(|c| c.get(2))
    };

    matcher.and_then(|c| c.as_str().parse().ok())
}

pub fn get_osu_mapset_id(msg: &str) -> Option<u32> {
    if let Ok(id) = msg.parse() {
        return Some(id);
    }

    if !msg.contains(OSU_BASE) {
        return None;
    }

    OSU_URL_MAPSET_OLD_MATCHER
        .captures(msg)
        .or_else(|| OSU_URL_MAP_NEW_MATCHER.captures(msg))
        .and_then(|c| c.get(1))
        .and_then(|c| c.as_str().parse().ok())
}

pub fn get_mods(msg: &str) -> Option<ModSelection> {
    let selection = if let Some(captures) = MOD_PLUS_MATCHER.captures(msg) {
        let mods = GameMods::from_str(captures.get(1)?.as_str()).ok()?;

        if msg.ends_with('!') {
            ModSelection::Exact(mods)
        } else {
            ModSelection::Include(mods)
        }
    } else if let Some(captures) = MOD_MINUS_MATCHER.captures(msg) {
        let mods = GameMods::from_str(captures.get(1)?.as_str()).ok()?;

        ModSelection::Exclude(mods)
    } else {
        return None;
    };

    Some(selection)
}

lazy_static::lazy_static! {
    static ref ROLE_ID_MATCHER: Regex = Regex::new(r"<@&(\d+)>").unwrap();

    static ref CHANNEL_ID_MATCHER: Regex = Regex::new(r"<#(\d+)>").unwrap();

    static ref MENTION_MATCHER: Regex = Regex::new(r"<@!?(\d+)>").unwrap();

    static ref OSU_URL_USER_MATCHER: Regex = Regex::new(r"^https://osu.ppy.sh/u(?:sers)?/(?:(\d+)|(\w+))$").unwrap();

    static ref OSU_URL_MAP_NEW_MATCHER: Regex = Regex::new(
        r"https://osu.ppy.sh/beatmapsets/(\d+)(?:(?:#(?:osu|mania|taiko|fruits)|<#\d+>)/(\d+))?"
    )
    .unwrap();

    static ref OSU_URL_MAP_OLD_MATCHER: Regex =
        Regex::new(r"https://osu.ppy.sh/b(?:eatmaps)?/(\d+)").unwrap();
    static ref OSU_URL_MAPSET_OLD_MATCHER: Regex =
        Regex::new(r"https://osu.ppy.sh/s/(\d+)").unwrap();

    static ref OSU_URL_MATCH_MATCHER: Regex =
        Regex::new(r"https://osu.ppy.sh/(?:community/matches|mp)/(\d+)").unwrap();

    static ref MOD_PLUS_MATCHER: Regex = Regex::new(r"^\+(\w+)!?$").unwrap();
    static ref MOD_MINUS_MATCHER: Regex = Regex::new(r"^-(\w+)!$").unwrap();

    static ref EMOJI_MATCHER: Regex = Regex::new(r"<(a?):([^:\n]+):(\d+)>").unwrap();

    static ref IGNORE_BADGE_MATCHER: Regex = Regex::new(r"^((?i)contrib|nomination|assessment|global|moderation|beatmap|spotlight|map|pending|aspire|elite|monthly|exemplary|outstanding|longstanding|idol[^@]+)").unwrap();

    static ref SEVEN_TWO_SEVEN: Regex = Regex::new("(?P<num>7[.,]?2[.,]?7)").unwrap();

    static ref OSU_SCORE_URL_MATCHER: Regex = Regex::new(r"https://osu.ppy.sh/scores/(osu|taiko|mania|fruits)/(\d+)").unwrap();

    pub static ref QUERY_SYNTAX_REGEX: Regex = Regex::new(r#"\b(?P<key>\w+)(?P<op>(:|=|(>|<)(:|=)?))(?P<value>("".*"")|(\S*))"#).unwrap();
}
