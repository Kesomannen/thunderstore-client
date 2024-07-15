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
pub use error::{Error, Result};
use models::{PackageExperimental, PackageVersionExperimental};
use std::{
    cmp,
    fmt::{self, Debug, Display},
    fs,
    hash::Hash,
    path::Path,
    str::FromStr,
};

mod error;
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

        let mut path = dir.as_ref().join(&version.repr);

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

/// A unique identifier for a package version, often formatted as `namespace-name-version`
/// and also known as a dependency string.
///
/// This struct is flexible and can be created in a number of ways:
/// ```
/// use thunderstore::VersionId;
///
/// let a = VersionId::new("BepInEx", "BepInExPack", "5.4.2100");
/// let b = "BepInEx-BepInExPack-5.4.2100".parse().unwrap();
/// let c = ("BepInEx", "BepInExPack", "5.4.2100").into();
/// let d = ("BepInEx", "BepInExPack", &semver::Version::new(5, 4, 2100)).into();
/// ```
///
/// Most methods on [`Client`] accept any type that implements [`IntoVersionId`],
/// which allows any of the above methods to be used interchangeably.
#[derive(Eq, Clone)]
pub struct VersionId {
    repr: String,
    name_start: usize,
    version_start: usize,
}

impl VersionId {
    pub fn new(namespace: &str, name: &str, version: &str) -> Self {
        let repr = format!("{}-{}-{}", namespace, name, version);
        let name_start = namespace.len() + 1;
        let version_start = name_start + name.len() + 1;
        Self {
            repr,
            name_start,
            version_start,
        }
    }

    pub fn namespace(&self) -> &str {
        &self.repr[..self.name_start - 1]
    }

    pub fn name(&self) -> &str {
        &self.repr[self.name_start..self.version_start - 1]
    }

    pub fn version(&self) -> &str {
        &self.repr[self.version_start..]
    }

    /// Returns an object that, when formatted with `{}`, will produce the URL path for this version.
    ///
    /// ## Example
    ///
    /// ```
    /// let id = thunderstore::VersionId::new("BepInEx", "BepInExPack", "5.4.2100");
    /// assert_eq!(id.path().to_string(), "BepInEx/BepInExPack/5.4.2100");
    /// ```
    pub fn path(&self) -> impl Display + '_ {
        VersionIdPath::new(self)
    }

    /// Consumes the [`VersionId`] and returns the underlying string, in the format `namespace-name-version`.
    pub fn into_string(self) -> String {
        self.repr
    }

    /// Returns a reference to the underlying string, formatted as `namespace-name-version`.
    pub fn as_str(&self) -> &str {
        &self.repr
    }
}

impl PartialEq for VersionId {
    fn eq(&self, other: &Self) -> bool {
        self.repr == other.repr
    }
}

impl Ord for VersionId {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.repr.cmp(&other.repr)
    }
}

impl PartialOrd for VersionId {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for VersionId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.repr.hash(state);
    }
}

impl AsRef<str> for VersionId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<VersionId> for String {
    fn from(id: VersionId) -> Self {
        id.repr
    }
}

impl Display for VersionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl Debug for VersionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VersionId({})", self.repr)
    }
}

impl TryFrom<String> for VersionId {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        let mut indices = value.match_indices('-').map(|(i, _)| i);

        let name_start = indices.next().ok_or(Error::InvalidPackageId)? + 1;

        let version_start = indices.next().ok_or(Error::InvalidPackageId)? + 1;

        Ok(Self {
            repr: value,
            name_start,
            version_start,
        })
    }
}

impl FromStr for VersionId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        s.to_string().try_into()
    }
}

impl From<(&str, &str, &str)> for VersionId {
    fn from((namespace, name, version): (&str, &str, &str)) -> Self {
        Self::new(namespace, name, version)
    }
}

impl From<(&str, &str, &semver::Version)> for VersionId {
    fn from((namespace, name, version): (&str, &str, &semver::Version)) -> Self {
        Self::new(namespace, name, &version.to_string())
    }
}

impl From<&PackageVersionExperimental> for VersionId {
    fn from(pkg: &PackageVersionExperimental) -> Self {
        Self::new(&pkg.namespace, &pkg.name, &pkg.version_number.to_string())
    }
}

impl From<&PackageExperimental> for VersionId {
    fn from(pkg: &PackageExperimental) -> Self {
        (&pkg.latest).into()
    }
}

struct VersionIdPath<'a> {
    id: &'a VersionId,
}

impl<'a> VersionIdPath<'a> {
    pub fn new(id: &'a VersionId) -> Self {
        Self { id }
    }
}

impl<'a> Display for VersionIdPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}",
            self.id.namespace(),
            self.id.name(),
            self.id.version()
        )
    }
}

/// A unique identifier for a package, often formatted as `namespace-name`.
///
/// This struct can be created in a number of ways:
/// ```
/// use thunderstore::PackageId;
///
/// let a = PackageId::new("BepInEx", "BepInExPack");
/// let b: PackageId = "BepInEx-BepInExPack".parse().unwrap();
/// let c: PackageId = ("BepInEx", "BepInExPack").into();
/// ```
///
/// Most methods on [`Client`] accept any type that implements [`IntoPackageId`],
/// which allows any of the above methods to be used interchangeably.
#[derive(Eq, Clone)]
pub struct PackageId {
    repr: String,
    name_start: usize,
}

impl PackageId {
    pub fn new(namespace: &str, name: &str) -> Self {
        let repr = format!("{}-{}", namespace, name);
        let name_start = namespace.len() + 1;
        Self { repr, name_start }
    }

    pub fn namespace(&self) -> &str {
        &self.repr[..self.name_start - 1]
    }

    pub fn name(&self) -> &str {
        &self.repr[self.name_start..]
    }

    /// Returns an object that, when formatted with `{}`, will produce the URL path for this package.
    ///
    /// ## Example
    ///
    /// ```
    /// let id = thunderstore::PackageId::new("BepInEx", "BepInExPack");
    /// assert_eq!(id.path().to_string(), "BepInEx/BepInExPack");
    /// ```
    pub fn path(&self) -> impl Display + '_ {
        PackageIdPath::new(self)
    }

    /// Consumes the [`PackageId`] and returns the underlying string, formatted as `namespace-name`.
    pub fn into_string(self) -> String {
        self.repr
    }

    /// Returns a reference to the underlying string, formatted as `namespace-name`.
    pub fn as_str(&self) -> &str {
        &self.repr
    }
}

impl PartialEq for PackageId {
    fn eq(&self, other: &Self) -> bool {
        self.repr == other.repr
    }
}

impl Ord for PackageId {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.repr.cmp(&other.repr)
    }
}

impl PartialOrd for PackageId {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for PackageId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.repr.hash(state);
    }
}

impl AsRef<str> for PackageId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<PackageId> for String {
    fn from(id: PackageId) -> Self {
        id.repr
    }
}

impl Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl Debug for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PackageId({})", self.repr)
    }
}

impl TryFrom<String> for PackageId {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        let mut indices = value.match_indices('-').map(|(i, _)| i);

        let name_start = indices.next().ok_or(Error::InvalidPackageId)? + 1;

        Ok(Self {
            repr: value,
            name_start,
        })
    }
}

impl FromStr for PackageId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        s.to_string().try_into()
    }
}

impl From<(&str, &str)> for PackageId {
    fn from((namespace, name): (&str, &str)) -> Self {
        Self::new(namespace, name)
    }
}

impl From<&PackageExperimental> for PackageId {
    fn from(pkg: &PackageExperimental) -> Self {
        Self::new(&pkg.namespace, &pkg.name)
    }
}

struct PackageIdPath<'a> {
    id: &'a PackageId,
}

impl<'a> PackageIdPath<'a> {
    pub fn new(id: &'a PackageId) -> Self {
        Self { id }
    }
}

impl<'a> Display for PackageIdPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.id.namespace(), self.id.name(),)
    }
}

impl From<&VersionId> for PackageId {
    fn from(id: &VersionId) -> Self {
        Self {
            repr: id.repr[..id.version_start].to_string(),
            name_start: id.name_start,
        }
    }
}

pub trait IntoVersionId {
    fn into_id(self) -> Result<VersionId>;
}

impl<T> IntoVersionId for T
where
    T: Into<VersionId>,
{
    fn into_id(self) -> Result<VersionId> {
        Ok(self.into())
    }
}

impl IntoVersionId for String {
    fn into_id(self) -> Result<VersionId> {
        self.try_into()
    }
}

impl IntoVersionId for &str {
    fn into_id(self) -> Result<VersionId> {
        self.parse()
    }
}

pub trait IntoPackageId {
    fn into_id(self) -> Result<PackageId>;
}

impl<T> IntoPackageId for T
where
    T: Into<PackageId>,
{
    fn into_id(self) -> Result<PackageId> {
        Ok(self.into())
    }
}

impl IntoPackageId for String {
    fn into_id(self) -> Result<PackageId> {
        self.try_into()
    }
}

impl IntoPackageId for &str {
    fn into_id(self) -> Result<PackageId> {
        self.parse()
    }
}
