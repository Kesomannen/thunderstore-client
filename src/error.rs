/// An error that can occur when interacting with the API.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// A non-specific network error.
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// JSON parse error.
    #[error("Failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    /// Base64 decode error.
    #[error("Failed to decode base64: {0}")]
    Base64(#[from] base64::DecodeError),

    /// The profile data was incorrectly formatted.
    #[error("Invalid profile data")]
    InvalidProfileData,

    /// A restricted enpoint was used, but the client's API token was missing or invalid.
    #[error("API token is missing or invalid")]
    ApiTokenInvalid,

    /// A 404 was returned by Thunderstore.
    #[error("Requested resource was not found")]
    NotFound,

    /// The package or version identifier was incorrectly formatted.
    #[error("Invalid package or version identifier")]
    InvalidIdent,
}

/// A [`Result`] alias where the error type is [`crate::Error`].
pub type Result<T> = std::result::Result<T, Error>;
