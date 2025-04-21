/// Represents an error that can occur when interacting with the API.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Semver error: {0}")]
    InvalidSemver(#[from] semver::Error),

    #[error("Failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Failed to decode base64: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("Invalid legacyprofile data")]
    InvalidProfileData,

    #[error("API token is missing or invalid")]
    ApiTokenInvalid,

    #[error("Requested resource was not found")]
    NotFound,

    #[error("Invalid package ID")]
    InvalidPackageId,
}

/// A [`Result`] alias where the error type is [`crate::Error`].
pub type Result<T> = std::result::Result<T, Error>;
