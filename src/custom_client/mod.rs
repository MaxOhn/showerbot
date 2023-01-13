use std::{fmt::Write, hash::Hash};

use bytes::Bytes;
use hashbrown::HashSet;
use http::{header::COOKIE, request::Builder as RequestBuilder, Response, StatusCode};
use hyper::{
    client::{connect::dns::GaiResolver, Client as HyperClient, HttpConnector},
    header::USER_AGENT,
    Body, Method, Request,
};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use leaky_bucket_lite::LeakyBucket;
use rosu_v2::prelude::{GameMode, GameMods};
use tokio::time::{sleep, Duration};

use crate::{
    core::BotConfig,
    util::{constants::OSU_BASE, ExponentialBackoff},
};

pub use self::{error::*, score::*};

use self::score::ScraperScores;

mod deser;
mod error;
mod score;

type ClientResult<T> = Result<T, CustomClientError>;

static MY_USER_AGENT: &str = env!("CARGO_PKG_NAME");

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
#[repr(u8)]
enum Site {
    OsuHiddenApi,
    OsuMapFile,
}

type Client = HyperClient<HttpsConnector<HttpConnector<GaiResolver>>, Body>;

pub struct CustomClient {
    client: Client,
    osu_session: &'static str,
    ratelimiters: [LeakyBucket; 2],
}

impl CustomClient {
    pub async fn new(config: &'static BotConfig) -> ClientResult<Self> {
        let connector = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http1()
            .build();

        let client = HyperClient::builder().build(connector);

        let ratelimiter = |per_second| {
            LeakyBucket::builder()
                .max(per_second)
                .tokens(per_second)
                .refill_interval(Duration::from_millis(1000 / per_second as u64))
                .refill_amount(1)
                .build()
        };

        let ratelimiters = [
            ratelimiter(2), // OsuHiddenApi
            ratelimiter(5), // OsuMapFile
        ];

        Ok(Self {
            client,
            osu_session: &config.tokens.osu_session,
            ratelimiters,
        })
    }

    async fn ratelimit(&self, site: Site) {
        self.ratelimiters[site as usize].acquire_one().await
    }

    async fn make_get_request(&self, url: impl AsRef<str>, site: Site) -> ClientResult<Bytes> {
        trace!("GET request of url {}", url.as_ref());

        let req = self
            .make_get_request_(url.as_ref(), site)
            .body(Body::empty())?;

        self.ratelimit(site).await;
        let response = self.client.request(req).await?;

        Self::error_for_status(response, url.as_ref()).await
    }

    fn make_get_request_(&self, url: impl AsRef<str>, site: Site) -> RequestBuilder {
        let req = Request::builder()
            .uri(url.as_ref())
            .method(Method::GET)
            .header(USER_AGENT, MY_USER_AGENT);

        match site {
            Site::OsuHiddenApi => req.header(COOKIE, format!("osu_session={}", self.osu_session)),
            _ => req,
        }
    }

    async fn error_for_status(
        response: Response<Body>,
        url: impl Into<String>,
    ) -> ClientResult<Bytes> {
        if response.status().is_client_error() || response.status().is_server_error() {
            Err(CustomClientError::Status {
                status: response.status(),
                url: url.into(),
            })
        } else {
            let bytes = hyper::body::to_bytes(response.into_body()).await?;

            Ok(bytes)
        }
    }

    // Retrieve the leaderboard of a map (national / global)
    // If mods contain DT / NC, it will do another request for the opposite
    // If mods dont contain Mirror and its a mania map, it will perform the
    // same requests again but with Mirror enabled
    pub async fn get_leaderboard(
        &self,
        map_id: u32,
        national: bool,
        mods: Option<GameMods>,
        mode: GameMode,
    ) -> ClientResult<Vec<ScraperScore>> {
        let mut scores = self._get_leaderboard(map_id, national, mods).await?;

        let non_mirror = mods
            .map(|mods| !mods.contains(GameMods::Mirror))
            .unwrap_or(true);

        // Check if another request for mania's MR is needed
        if mode == GameMode::Mania && non_mirror {
            let mods = match mods {
                None => Some(GameMods::Mirror),
                Some(mods) => Some(mods | GameMods::Mirror),
            };

            let mut new_scores = self._get_leaderboard(map_id, national, mods).await?;
            scores.append(&mut new_scores);
            scores.sort_unstable_by(|a, b| b.score.cmp(&a.score));
            let mut uniques = HashSet::with_capacity(50);
            scores.retain(|s| uniques.insert(s.user_id));
            scores.truncate(50);
        }

        // Check if DT / NC is included
        let mods = match mods {
            Some(mods) if mods.contains(GameMods::DoubleTime) => Some(mods | GameMods::NightCore),
            Some(mods) if mods.contains(GameMods::NightCore) => {
                Some((mods - GameMods::NightCore) | GameMods::DoubleTime)
            }
            Some(_) | None => None,
        };

        // If DT / NC included, make another request
        if mods.is_some() {
            if mode == GameMode::Mania && non_mirror {
                let mods = mods.map(|mods| mods | GameMods::Mirror);
                let mut new_scores = self._get_leaderboard(map_id, national, mods).await?;
                scores.append(&mut new_scores);
            }

            let mut new_scores = self._get_leaderboard(map_id, national, mods).await?;
            scores.append(&mut new_scores);
            scores.sort_unstable_by(|a, b| b.score.cmp(&a.score));
            let mut uniques = HashSet::with_capacity(50);
            scores.retain(|s| uniques.insert(s.user_id));
            scores.truncate(50);
        }

        Ok(scores)
    }

    // Retrieve the leaderboard of a map (national / global)
    async fn _get_leaderboard(
        &self,
        map_id: u32,
        national: bool,
        mods: Option<GameMods>,
    ) -> ClientResult<Vec<ScraperScore>> {
        let mut url = format!("{OSU_BASE}beatmaps/{map_id}/scores?");

        if national {
            url.push_str("type=country");
        }

        if let Some(mods) = mods {
            if mods.is_empty() {
                url.push_str("&mods[]=NM");
            } else {
                for m in mods.iter() {
                    let _ = write!(url, "&mods[]={m}");
                }
            }
        }

        let bytes = self.make_get_request(url, Site::OsuHiddenApi).await?;

        let scores: ScraperScores = serde_json::from_slice(&bytes)
            .map_err(|e| CustomClientError::parsing(e, &bytes, ErrorKind::Leaderboard))?;

        Ok(scores.get())
    }

    pub async fn get_map_file(&self, map_id: u32) -> ClientResult<Bytes> {
        let url = format!("{OSU_BASE}osu/{map_id}");
        let backoff = ExponentialBackoff::new(2).factor(500).max_delay(10_000);
        const ATTEMPTS: usize = 10;

        for (duration, i) in backoff.take(ATTEMPTS).zip(1..) {
            let result = self.make_get_request(&url, Site::OsuMapFile).await;

            if matches!(&result, Err(CustomClientError::Status { status, ..}) if *status == StatusCode::TOO_MANY_REQUESTS)
                || matches!(&result, Ok(bytes) if bytes.starts_with(b"<html>"))
            {
                debug!("Request beatmap retry attempt #{i} | Backoff {duration:?}");
                sleep(duration).await;
            } else {
                return result;
            }
        }

        Err(CustomClientError::MapFileRetryLimit(map_id))
    }
}
