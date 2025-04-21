use crate::{util, IntoVersionId, Result};
use bytes::Bytes;
use reqwest::Method;
use serde::{de::DeserializeOwned, Serialize};

const DEFAULT_BASE_URL: &str = "https://thunderstore.io";

/// A client for interacting with the Thunderstore API.
///
/// The easiest way to create a client is to use the [`Client::new`] method.
/// If you need more control over the client's configuration, use the [`Client::builder`] method instead.
pub struct Client {
    pub(crate) base_url: String,
    pub(crate) client: reqwest::Client,
    pub(crate) token: Option<String>,
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

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn set_base_url(&mut self, base_url: impl Into<String>) {
        self.base_url = base_url.into();
    }

    pub fn set_default_base_url(&mut self) {
        self.base_url = DEFAULT_BASE_URL.to_string();
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub fn clear_token(&mut self) {
        self.token = None;
    }

    pub fn set_token(&mut self, token: impl Into<String>) {
        self.token = Some(token.into());
    }

    pub(crate) async fn request(
        &self,
        method: reqwest::Method,
        url: impl reqwest::IntoUrl,
        body: Option<reqwest::Body>,
        headers: Option<reqwest::header::HeaderMap>,
    ) -> Result<reqwest::Response> {
        let mut request = self.client.request(method, url);

        if let Some(body) = body {
            request = request.body(body);
        }

        if let Some(headers) = headers {
            request = request.headers(headers);
        }

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        util::map_reqwest_response(request.send().await)
    }

    pub(crate) async fn get(&self, url: impl reqwest::IntoUrl) -> Result<reqwest::Response> {
        self.request(Method::GET, url, None, None).await
    }

    pub(crate) async fn get_json<T>(&self, url: impl reqwest::IntoUrl) -> Result<T>
    where
        T: DeserializeOwned,
    {
        Ok(self.get(url).await?.json().await?)
    }

    pub(crate) async fn post(
        &self,
        url: impl reqwest::IntoUrl,
        body: impl Into<reqwest::Body>,
        headers: Option<reqwest::header::HeaderMap>,
    ) -> Result<reqwest::Response> {
        self.request(Method::POST, url, Some(body.into()), headers)
            .await
    }

    pub(crate) async fn post_json<T>(
        &self,
        url: impl reqwest::IntoUrl,
        body: &T,
    ) -> Result<reqwest::Response>
    where
        T: Serialize,
    {
        let headers = util::header_map([("Content-Type", "application/json")]);
        self.post(url, serde_json::to_string(body)?, Some(headers))
            .await
    }

    /// Downloads a package from Thunderstore.
    /// The result is a ZIP archive containing the contents of the package.
    pub async fn download(&self, version: impl IntoVersionId<'_>) -> Result<Bytes> {
        let url = format!(
            "{}/package/download/{}",
            self.base_url,
            version.into_id()?.path()
        );
        let response = self.get(url).await?.bytes().await?;

        Ok(response)
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
    pub fn use_dev_repo(self) -> Self {
        self.with_base_url("https://thunderstore.dev".to_owned())
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
