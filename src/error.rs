use std::fmt;

/// Application errors mapped to the shared exit-code contract
/// (see baby-registry/docs/cli-conventions.md).
#[derive(Debug)]
pub enum AppError {
    /// No matching product / empty result set.
    NotFound(String),
    /// Network or upstream (non-2xx) failure.
    Network(String),
    /// HTTP 429 from Target.
    RateLimited,
    /// Anything else.
    Other(String),
}

impl AppError {
    pub fn exit_code(&self) -> i32 {
        match self {
            AppError::Other(_) => 1,
            AppError::NotFound(_) => 4,
            AppError::Network(_) => 5,
            AppError::RateLimited => 6,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(m) => write!(f, "not found: {m}"),
            AppError::Network(m) => write!(f, "network/upstream error: {m}"),
            AppError::RateLimited => {
                write!(f, "rate limited by Target (HTTP 429) — slow down and retry")
            }
            AppError::Other(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::Network(e.to_string())
    }
}
