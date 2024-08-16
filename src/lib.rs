//! A library for interacting with the Thunderstore API.
//!
//! The main struct is [`Client`], which provides methods for fetching, downloading and publishing packages.
//! The easiest way to get up and running is to use the [`Client::new`] method, which creates a client with the default configuration.
//! If you need more control over the client's configuration, use the [`Client::builder`] method instead (see [`ClientBuilder`]).
//!
//! Some methods, including uploading and submitting packages, require an API token to be set on the client.
//! You can set this token using the [`ClientBuilder::with_token`] method.
//!
//! # Examples
//!
//! ```no_run
//! #[tokio::main]
//! async fn main() -> thunderstore::Result<()> {
//!     let client = thunderstore::Client::builder()
//!         .with_token("tss_XXX")
//!         .build()?;
//!
//!     let package = client.get_package(("Kesomannen", "GaleModManager")).await?;
//!     client.download_to_dir(&package.latest, r"C:\Users\bobbo\Downloads").await?;
//!
//!     client.publish_file(
//!         "CoolMod.zip",
//!         PackageMetadata::new("Kesomannen", ["lethal-company"])
//!             .with_global_categories(["tools"])
//!             .with_categories("lethal-company", ["serverside"])
//!     ).await?;
//! }
//! ```

use bytes::Bytes;
use std::{fmt::Debug, fs, path::Path};

pub use error::{Error, Result};
pub use id::{IntoPackageId, IntoVersionId, PackageId, VersionId};

mod error;
mod id;

pub mod experimental;
pub mod models;
pub mod schema;
pub mod usermedia;
pub mod v1;
pub mod wiki;

#[cfg(test)]
mod tests;

const DEFAULT_BASE_URL: &str = "https://thunderstore.io";

/// A client for interacting with the Thunderstore API.
///
/// The easiest way to create a client is to use the [`Client::new`] method.
/// If you need more control over the client's configuration, use the [`Client::builder`] method instead.
pub struct Client {
    base_url: String,
    client: reqwest::Client,
    token: Option<String>,
}

impl Client {
    /// Creates a new client with the default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a [`ClientBuilder`] to configure a new client.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    fn auth_request(
        &self,
        method: reqwest::Method,
        url: impl reqwest::IntoUrl,
    ) -> Result<reqwest::RequestBuilder> {
        if let Some(token) = self.token.as_ref() {
            Ok(self.client.request(method, url).bearer_auth(token))
        } else {
            Err(Error::ApiTokenRequired)
        }
    }

    /// Downloads a package from Thunderstore.
    /// The resulting bytes represent a ZIP archive containing the contents of the package.
    ///
    /// If you want to save the package to a file, use the [`Client::download_to_file`] or
    /// [`Client::download_to_dir`] methods instead.
    pub async fn download(&self, version: impl IntoVersionId) -> Result<Bytes> {
        let url = format!(
            "{}/package/download/{}/",
            self.base_url,
            version.into_id()?.path()
        );
        let response = self.client.get(&url).send().await.handle()?.bytes().await?;

        Ok(response)
    }

    /// Downloads a package and saves it to a file.
    pub async fn download_to_file(
        &self,
        version: impl IntoVersionId,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let data = self.download(version).await?;
        fs::write(path, data).map_err(Error::Io)
    }

    /// Downloads a package and saves it to a directory.
    /// The file will be saved with the format `{dir}/{namespace}-{name}-{version}.zip`.
    ///
    /// ## Example
    ///
    /// ```
    /// let client = thunderstore::Client::new();
    ///
    /// client.download_to_dir(
    ///     "no00ob-LCSoundTool-1.5.1",
    ///     r"C:\Users\goober\Downloads"
    /// ).await?;
    ///
    /// let package = client.get_package(("BepInEx", "BepInExPack")).await?;
    /// client.download_to_dir(&package.latest, r"C:\Users\goober\Downloads").await?;
    /// ```
    pub async fn download_to_dir(
        &self,
        version: impl IntoVersionId,
        dir: impl AsRef<Path>,
    ) -> Result<()> {
        let version = version.into_id()?;

        let mut path = dir.as_ref().join(version.as_str());

        // the final version component might be treated as a file extension,
        // so we can't do path.set_extension(), since that replaces the existing one
        if let Some(ext) = path.extension() {
            let mut new_ext = ext.to_os_string();
            new_ext.push(".zip");
            path.set_extension(new_ext);
        }

        self.download_to_file(version, path).await
    }
}

impl Default for Client {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            client: reqwest::Client::new(),
            token: None,
        }
    }
}

trait ResponseExt {
    fn handle(self) -> Result<reqwest::Response>;
}

impl ResponseExt for reqwest::Result<reqwest::Response> {
    fn handle(self) -> Result<reqwest::Response> {
        match self.and_then(|res| res.error_for_status()) {
            Ok(res) => Ok(res),
            Err(err) => match err.status() {
                Some(reqwest::StatusCode::UNAUTHORIZED) => Err(Error::ApiTokenInvalid),
                Some(reqwest::StatusCode::NOT_FOUND) => Err(Error::NotFound),
                _ => Err(Error::Reqwest(err)),
            },
        }
    }
}

/// A builder for configuring a [`Client`] instance.
#[derive(Debug, Default)]
pub struct ClientBuilder {
    base_url: Option<String>,
    client: Option<reqwest::Client>,
    token: Option<String>,
}

impl ClientBuilder {
    /// Creates a new client builder with the default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the base URL for requests. Defaults to `https://thunderstore.io`.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Sets the client to use Thunderstore's staging repository instead of the main one.
    ///
    /// Equivalent to calling `with_base_url("https://thunderstore.dev")`.
    pub fn use_dev_repo(mut self) -> Self {
        self.base_url = Some("https://thunderstore.dev".to_owned());
        self
    }

    /// Sets the network client to use for requests.
    /// See the [`reqwest::Client`] documentation for more information.
    pub fn with_client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Sets the API token to use for requests.
    ///
    /// This is required for some actions, such as uploading packages.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Builds a client with the configured options.
    pub fn build(self) -> Result<Client> {
        Ok(Client {
            base_url: self
                .base_url
                .unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            client: self.client.unwrap_or_default(),
            token: self.token,
        })
    }
}
