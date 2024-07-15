use crate::{models::*, Client, Error, ResponseExt, Result};
use bytes::Bytes;
use futures_util::future::join_all;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, path::Path};
use tokio::fs;
use uuid::Uuid;

impl Client {
    /// Initiates a new package upload. 
    /// 
    /// - `name` corresponds to the name of the package and may only contain alphanumeric
    /// characters and underscores.
    /// 
    /// - `size` must be the package's size in bytes.
    ///
    /// This method returns a [`UserMediaInitiateUploadResponse`] which contains a unique UUID for the upload,
    /// which is used to identify the package throughout the upload process.
    /// 
    /// The response also contains a list of URLs to which the file should be uploaded, using HTTP PUT.
    /// Each upload URL responds with an ETag header, which should be used to finalize the upload.
    ///
    /// Alternatively, you can use [`Client::publish`] to upload and submit a package in one go.
    /// 
    /// This method requires a valid API token on the client.
    /// 
    /// ## Example
    /// 
    /// ```no_run
    /// use thunderstore::{Client, models::{UploadPartUrl, CompletedPart}};
    ///
    /// let client = Client::builder().with_token("tss_XXX").build()?;
    ///
    /// let path = "path/to/your/package.zip";
    /// let size = std::fs::metadata(path)?.len();
    /// let response = client.initiate_upload("MyCoolMod", size).await?;
    ///
    /// let uuid = response.user_media.uuid.unwrap();
    /// let parts = Vec::new();
    ///
    /// for UploadPartUrl { url, part_number, offset, length } in response.upload_urls {
    ///    // Read `length` bytes from `offset` in the file
    ///    // and make a PUT request to `url` with the data
    ///
    ///    // The response will return an ETag header, which is needed to complete the upload
    ///    parts.push(CompletedPart { tag: todo!(), part_number });
    /// 
    ///    // These requests should preferably be done concurrently to decrease upload time
    /// }
    /// 
    /// client.finish_upload(uuid, parts).await?;
    /// ```
    pub async fn initiate_upload(
        &self,
        name: impl Into<String>,
        size: u64,
    ) -> Result<UserMediaInitiateUploadResponse> {
        let url = self.usermedia_url("initiate-upload");
        let response = self
            .auth_request(Method::POST, url)?
            .json(&UserMediaInitiateUploadParams {
                filename: name.into(),
                file_size_bytes: size,
            })
            .send()
            .await
            .handle()?
            .json()
            .await?;

        Ok(response)
    }

    /// Aborts an ongoing upload.
    ///
    /// This method requires a valid API token on the client.
    pub async fn abort_upload(&self, uuid: Uuid) -> Result<UserMedia> {
        let url = self.usermedia_url(format_args!("{}/abort-upload", uuid));

        let response = self
            .auth_request(Method::POST, url)?
            .send()
            .await
            .handle()?
            .json()
            .await?;

        Ok(response)
    }

    /// Finalizes an upload to Thunderstore. Requires the UUID of the upload and a list
    /// of [`CompletedPart`] objects, which contain the ETag of each part of the upload.
    ///
    /// Note that this will not publish the package, only finish the upload process.
    /// To submit the package, use the [`Client::submit_submission`] method as well.
    ///
    /// This method requires a valid API token on the client.
    pub async fn finish_upload(&self, uuid: Uuid, parts: Vec<CompletedPart>) -> Result<UserMedia> {
        let url = self.usermedia_url(format_args!("{}/finish-upload", uuid));

        let response = self
            .auth_request(Method::POST, url)?
            .json(&UserMediaFinishUploadParams { parts })
            .send()
            .await
            .handle()?
            .json()
            .await?;

        Ok(response)
    }

    /// Uploads and submits a package.
    /// 
    /// - `name` may only contain alphanumeric characters and underscores.
    ///
    /// This method requires a valid API token on the client.
    pub async fn publish(
        &self,
        name: impl Into<String>,
        data: Vec<u8>,
        metadata: PackageMetadata,
    ) -> Result<PackageSubmissionResult> {
        let bytes = Bytes::from(data);
        let response = self.initiate_upload(name, bytes.len() as u64).await?;

        let uuid = response.user_media.uuid.ok_or(Error::NoUploadUuidGiven)?;

        let chunks = response
            .upload_urls
            .into_iter()
            .map(|part| tokio::spawn(upload_chunk(self.client.clone(), part, bytes.clone())));

        let parts = join_all(chunks)
            .await
            .into_iter()
            .flatten()
            .collect::<Result<Vec<_>>>()?;

        self.finish_upload(uuid, parts).await?;
        self.submit_package(uuid, metadata).await
    }

    /// Uploads and submits a package.
    /// The name of the package is derived from the file name.
    ///
    /// This method requires a valid API token on the client.
    pub async fn publish_file(
        &self,
        path: impl AsRef<Path>,
        metadata: PackageMetadata,
    ) -> Result<PackageSubmissionResult> {
        let file_name = path
            .as_ref()
            .with_extension("")
            .to_string_lossy()
            .to_string();

        let data = fs::read(path).await?;
        self.publish(file_name, data, metadata).await
    }

    pub(crate) fn usermedia_url(&self, tail: impl Display) -> String {
        format!("{}/api/experimental/usermedia/{}/", self.base_url, tail)
    }
}

async fn upload_chunk(
    client: reqwest::Client,
    part: UploadPartUrl,
    bytes: Bytes,
) -> Result<CompletedPart> {
    let slice = bytes.slice(part.offset as usize..(part.offset + part.length) as usize);

    let response = client.put(&part.url).body(slice).send().await?.error_for_status()?;

    let tag = response
        .headers()
        .get("ETag")
        .expect("no ETag in response")
        .to_str()
        .expect("ETag is not valid ascii")
        .to_owned();

    Ok(CompletedPart {
        tag,
        part_number: part.part_number,
    })
}

/// Metadata for a package submission.
///
/// Use [`PackageMetadata::new`] to create a new instance, then customize it using builder methods.
///
/// ## Example
///
/// ```
/// use thunderstore::usermedia::PackageMetadata;
///
/// PackageMetadata::new("Kesomannen", ["content-warning", "lethal-company"])
///     .with_global_categories(["mods"])
///     .with_categories("lethal-company", ["audio", "serverside"])
///     .with_categories("content-warning", ["audio"]);
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct PackageMetadata {
    #[serde(rename = "author_name")]
    author: String,
    #[serde(rename = "categories")]
    global_categories: Vec<String>,
    communities: Vec<String>,
    has_nsfw_content: bool,
    #[serde(rename = "community_categories")]
    categories: HashMap<String, Vec<String>>,
    pub(crate) upload_uuid: Option<Uuid>,
}

impl PackageMetadata {
    /// Creates a new package metadata object.
    ///
    /// - `author` is the author of the package. This must be the same as the thunderstore team's name.
    /// - `communities` is a list of communities to publish the package to, referred to by their slug.
    ///
    /// You can provide further configuration using associated builder methods.
    /// See [`PackageMetadata`] for examples.
    pub fn new<C>(author: impl ToString, communities: impl IntoIterator<Item = C>) -> Self
    where
        C: Into<String>,
    {
        Self {
            author: author.to_string(),
            global_categories: Vec::new(),
            communities: communities.into_iter().map(Into::into).collect(),
            has_nsfw_content: false,
            upload_uuid: None,
            categories: HashMap::new(),
        }
    }

    /// Adds a list of site-wide categories to the package.
    ///
    /// Categories are referred to by their slug, *not* the display name!
    ///
    /// ## Example
    ///
    /// ```
    /// use thunderstore::usermedia::PackageMetadata;
    ///
    /// PackageMetadata::new("author", ["among-us"])
    ///     .with_global_categories([
    ///         "mods",
    ///         "modpacks",
    ///         "libraries",
    ///         "asset-replacements"
    ///     ]);
    /// ```
    pub fn with_global_categories<C>(mut self, categories: impl IntoIterator<Item = C>) -> Self
    where
        C: Into<String>,
    {
        self.global_categories
            .extend(categories.into_iter().map(Into::into));
        self
    }

    /// Adds a community to publish the package to.
    ///
    /// Communities are referred to by their slug, which is usually in kebab-case.
    ///
    /// ## Example
    ///
    /// ```
    /// use thunderstore::usermedia::PackageMetadata;
    ///
    /// PackageMetadata::new("author", ["among-us"])
    ///     .in_community("lethal-company")
    ///     .in_community("riskofrain2");
    /// ```
    pub fn in_community(mut self, community: impl Into<String>) -> Self {
        self.communities.push(community.into());
        self
    }

    /// Adds a list of communities to publish the package to.
    ///
    /// Communities are referred to by their slug, which is usually in kebab-case.
    ///
    /// ## Example
    ///
    /// ```
    /// use thunderstore::usermedia::PackageMetadata;
    ///
    /// PackageMetadata::new("author", ["among-us"])
    ///     .in_global_communities([
    ///         "lethal-company",
    ///         "riskofrain2",
    ///         "valheim",
    ///         "content-warning"
    ///     ]);
    /// ```
    pub fn in_communities<C>(mut self, communities: impl IntoIterator<Item = C>) -> Self
    where
        C: Into<String>,
    {
        self.communities
            .extend(communities.into_iter().map(Into::into));
        self
    }

    /// Specifies whether the package contains NSFW content.
    pub fn has_nsfw_content(mut self, value: bool) -> Self {
        self.has_nsfw_content = value;
        self
    }

    /// Adds a list of community-specific categories to the package.
    ///
    /// Categories are referred to by their slug, *not* the display name!
    ///
    /// ## Example
    ///
    /// ```
    /// use thunderstore::usermedia::PackageMetadata;
    ///
    /// PackageMetadata::new("author", ["lethal-company", "content-warning"])
    ///     .with_categories("lethal-company", ["items"])
    ///     .with_categories("content-warning", ["emotes", "camera"]);
    pub fn with_categories<C>(
        mut self,
        community: impl Into<String>,
        categories: impl IntoIterator<Item = C>,
    ) -> Self
    where
        C: Into<String>,
    {
        self.categories
            .entry(community.into())
            .or_default()
            .extend(categories.into_iter().map(Into::into));
        self
    }
}
