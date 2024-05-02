use std::fmt;

use http::StatusCode;
use serde_json::Error;

#[derive(Debug, thiserror::Error)]
pub enum CustomClientError {
    #[error("failed to create header value")]
    InvalidHeader(#[from] hyper::header::InvalidHeaderValue),
    #[error("http error")]
    Http(#[from] hyper::http::Error),
    #[error("hyper error")]
    Hyper(#[from] hyper::Error),
    #[error("could not deserialize {kind}: {body}")]
    Parsing {
        body: String,
        kind: ErrorKind,
        #[source]
        source: Error,
    },
    #[error("reached retry limit and still failed")]
    RetryLimit,
    #[error("failed with status code {status} when requesting {url}")]
    Status { status: StatusCode, url: String },
}

impl CustomClientError {
    pub fn parsing(source: Error, bytes: &[u8], kind: ErrorKind) -> Self {
        Self::Parsing {
            body: String::from_utf8_lossy(bytes).into_owned(),
            source,
            kind,
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Leaderboard,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self {
            Self::Leaderboard => "leaderboard",
        };

        f.write_str(kind)
    }
}
