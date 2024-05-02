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
use rosu_v2::{
    model::score::Score,
    prelude::{GameModIntermode, GameMode, GameModsIntermode},
};
use tokio::time::{sleep, Duration};

use crate::{
    core::BotConfig,
    util::{constants::OSU_BASE, ExponentialBackoff},
};

pub use self::error::*;

use self::scores::Scores;

mod error;
mod scores;

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

        let ratelimiter = |per_ten_seconds| {
            LeakyBucket::builder()
                .max(1)
                .tokens(1)
                .refill_interval(Duration::from_millis(10_000 / per_ten_seconds as u64))
                .refill_amount(1)
                .build()
        };

        let ratelimiters = [
            ratelimiter(4),  // OsuHiddenApi
            ratelimiter(20), // OsuMapFile
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

    async fn make_get_request(&self, url: &str, site: Site) -> ClientResult<Bytes> {
        const ATTEMPTS: usize = 10;

        trace!("GET request of url {url}");

        let backoff = ExponentialBackoff::new(2).factor(500).max_delay(10_000);

        for (duration, i) in backoff.take(ATTEMPTS).zip(1..) {
            let req = self.make_get_request_(url, site).body(Body::empty())?;
            self.ratelimit(site).await;
            let response = self.client.request(req).await?;
            let res = Self::error_for_status(response, url).await;

            if let Err(CustomClientError::Status {
                status: StatusCode::TOO_MANY_REQUESTS,
                ..
            }) = res
            {
                debug!("Retry attempt #{i} for {url} | Backoff {duration:?}");
                sleep(duration).await;
            } else {
                return res;
            }
        }

        Err(CustomClientError::RetryLimit)
    }

    fn make_get_request_(&self, url: &str, site: Site) -> RequestBuilder {
        let req = Request::builder()
            .uri(url)
            .method(Method::GET)
            .header(USER_AGENT, MY_USER_AGENT);

        match site {
            Site::OsuHiddenApi => req.header(COOKIE, format!("osu_session={}", self.osu_session)),
            _ => req,
        }
    }

    async fn error_for_status(response: Response<Body>, url: &str) -> ClientResult<Bytes> {
        if response.status().is_client_error() || response.status().is_server_error() {
            Err(CustomClientError::Status {
                status: response.status(),
                url: url.to_owned(),
            })
        } else {
            let bytes = hyper::body::to_bytes(response.into_body()).await?;

            Ok(bytes)
        }
    }

    // Retrieve the national leaderboard of a map
    // If mods contain DT / NC, it will do another request for the opposite
    // If mods dont contain Mirror and its a mania map, it will perform the
    // same requests again but with Mirror enabled
    pub async fn get_leaderboard(
        &self,
        map_id: u32,
        mods: Option<&GameModsIntermode>,
        mode: GameMode,
    ) -> ClientResult<Vec<Score>> {
        let mut scores = self.get_leaderboard_(map_id, mods).await?;

        let non_mirror = mods
            .map(|mods| !mods.contains(GameModIntermode::Mirror))
            .unwrap_or(true);

        // Check if another request for mania's MR is needed
        if mode == GameMode::Mania && non_mirror {
            let mods = match mods {
                None => Some(rosu_v2::mods!(MR)),
                Some(mods) => Some(mods.clone() | GameModIntermode::Mirror),
            };

            let mut new_scores = self.get_leaderboard_(map_id, mods.as_ref()).await?;
            scores.append(&mut new_scores);
            scores.sort_unstable_by(|a, b| b.score.cmp(&a.score));
            let mut uniques = HashSet::with_capacity(50);
            scores.retain(|s| uniques.insert(s.user_id));
            scores.truncate(50);
        }

        // Check if DT / NC is included
        let mods = match mods {
            Some(mods) if mods.contains(GameModIntermode::DoubleTime) => {
                Some(mods.clone() | GameModIntermode::Nightcore)
            }
            Some(mods) if mods.contains(GameModIntermode::Nightcore) => {
                Some((mods.clone() - GameModIntermode::Nightcore) | GameModIntermode::DoubleTime)
            }
            Some(_) | None => None,
        };

        // If DT / NC included, make another request
        if mods.is_some() {
            if mode == GameMode::Mania && non_mirror {
                let mods = mods
                    .as_ref()
                    .map(|mods| mods.clone() | GameModIntermode::Mirror);
                let mut new_scores = self.get_leaderboard_(map_id, mods.as_ref()).await?;
                scores.append(&mut new_scores);
            }

            let mut new_scores = self.get_leaderboard_(map_id, mods.as_ref()).await?;
            scores.append(&mut new_scores);
            scores.sort_unstable_by(|a, b| b.score.cmp(&a.score));
            let mut uniques = HashSet::with_capacity(50);
            scores.retain(|s| uniques.insert(s.user_id));
            scores.truncate(50);
        }

        Ok(scores)
    }

    // Retrieve the national leaderboard of a map
    async fn get_leaderboard_(
        &self,
        map_id: u32,
        mods: Option<&GameModsIntermode>,
    ) -> ClientResult<Vec<Score>> {
        let mut url = format!("{OSU_BASE}beatmaps/{map_id}/scores?type=country");

        if let Some(mods) = mods {
            if mods.is_empty() {
                url.push_str("&mods[]=NM");
            } else {
                for m in mods.iter() {
                    let _ = write!(url, "&mods[]={m}");
                }
            }
        }

        let bytes = self.make_get_request(&url, Site::OsuHiddenApi).await?;

        let scores: Scores = serde_json::from_slice(&bytes)
            .map_err(|e| CustomClientError::parsing(e, &bytes, ErrorKind::Leaderboard))?;

        Ok(scores.get())
    }

    pub async fn get_map_file(&self, map_id: u32) -> ClientResult<Bytes> {
        let url = format!("{OSU_BASE}osu/{map_id}");
        let bytes = self.make_get_request(&url, Site::OsuMapFile).await?;

        // On invalid response, retry once more
        if bytes.starts_with(b"<html>") {
            self.make_get_request(&url, Site::OsuMapFile).await
        } else {
            Ok(bytes)
        }
    }
}
