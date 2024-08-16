use crate::{
    models::*, usermedia::PackageMetadata, Client, Error, IntoPackageId, IntoVersionId,
    ResponseExt, Result,
};
use base64::{prelude::BASE64_STANDARD, Engine};
use reqwest::Method;
use std::{fmt::Display, path::Path};
use tokio::fs;
use uuid::Uuid;

const PROFILE_DATA_PREFIX: &str = "#r2modman\n";

impl Client {
    /// Fetches a list of all packages on Thunderstore.
    pub async fn get_package_index(&self) -> Result<Vec<PackageIndexEntry>> {
        let url = self.experimental_url("package-index");

        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        response
            .lines()
            .map(|line| serde_json::from_str(line).map_err(Error::Json))
            .collect()
    }

    /// Fetches information about a single package.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// let client = thunderstore::Client::new();
    ///
    /// let a = client.get_package(("Kesomannen", "GaleModManager")).await?;
    /// let b = client.get_package("Kesomannen-GaleModManager").await?;
    ///
    /// assert_eq!(a, b);
    /// ```
    pub async fn get_package(&self, id: impl IntoPackageId) -> Result<Package> {
        let url = self.experimental_url(format_args!("package/{}", id.into_id()?.path()));
        let response = self.client.get(&url).send().await.handle()?.json().await?;
        Ok(response)
    }

    /// Fetches information about a specific version of a package.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// let client = thunderstore::Client::new();
    ///
    /// let a = client.get_version(("Kesomannen", "GaleModManager", "0.6.0")).await?;
    /// let b = client.get_version("Kesomannen-GaleModManager-0.6.0").await?;
    ///
    /// assert_eq!(a, b);
    /// ```
    pub async fn get_version(&self, id: impl IntoVersionId) -> Result<PackageVersion> {
        let url = self.experimental_url(format_args!("package/{}", id.into_id()?.path()));
        let response = self.client.get(&url).send().await.handle()?.json().await?;
        Ok(response)
    }

    /// Fetches the changelog for a specific version of a package.
    /// The changelog is returned as a markdown string.
    ///
    /// Note that a package may not have a changelog, in which case [`Error::NotFound`] is returned.
    pub async fn get_changelog(&self, id: impl IntoVersionId) -> Result<String> {
        let url = self.experimental_url(format_args!("package/{}/changelog", id.into_id()?.path()));
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .handle()?
            .json::<MarkdownResponse>()
            .await?;

        Ok(response.markdown)
    }

    /// Fetches the readme for a specific version of a package.
    /// The readme is returned as a markdown string.
    pub async fn get_readme(&self, id: impl IntoVersionId) -> Result<String> {
        let url = self.experimental_url(format_args!("package/{}/readme", id.into_id()?.path()));
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .handle()?
            .json::<MarkdownResponse>()
            .await?;

        Ok(response.markdown)
    }

    /// Renders a markdown string to HTML.
    pub async fn render_markdown(&self, markdown: impl ToString) -> Result<String> {
        let url = self.experimental_url("frontend/render-markdown");
        let response = self
            .client
            .post(&url)
            .json(&RenderMarkdownParams {
                markdown: markdown.to_string(),
            })
            .send()
            .await?
            .json::<RenderMarkdownResponse>()
            .await?;

        Ok(response.html)
    }

    /// Creates a profile with the given data and returns its key.
    ///
    /// The data is expected to be a ZIP archive containing a `mods.yml` file and
    /// any addition configuration files or directories. However, any arbitrary
    /// data is allowed, but will likely fail to import correctly in mod managers.
    ///
    /// The key returned is used to retrieve the profile with [`Client::get_profile`].
    pub async fn create_profile(&self, data: impl AsRef<[u8]>) -> Result<Uuid> {
        let mut base64 = String::from(PROFILE_DATA_PREFIX);
        base64.push_str(&BASE64_STANDARD.encode(data));

        let url = self.experimental_url("legacyprofile/create");

        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/octet-stream")
            .body(base64)
            .send()
            .await?
            .error_for_status()?
            .json::<LegacyProfileCreateResponse>()
            .await?;

        Ok(response.key)
    }

    /// Downloads a profile with the given key.
    ///
    /// The returned data is usually a ZIP archive containing a mods.yml file and
    /// any additional configuration files. However, any arbitrary data is allowed
    /// by Thunderstore.
    pub async fn get_profile(&self, key: Uuid) -> Result<Vec<u8>> {
        let url = self.experimental_url(format_args!("legacyprofile/get/{}", key));

        let response = self.client.get(url).send().await.handle()?.text().await?;

        match response.strip_prefix(PROFILE_DATA_PREFIX) {
            Some(data) => BASE64_STANDARD.decode(data).map_err(Error::Base64),
            None => Err(Error::InvalidProfileData),
        }
    }

    /// Downloads a profile with the given key and saves it to a file.
    ///
    /// The resulting file is usually a ZIP archive containing a mods.yml file and
    /// any additional configuration files. However, any arbitrary data is allowed
    /// by Thunderstore.
    pub async fn save_profile(&self, key: Uuid, path: impl AsRef<Path>) -> Result<()> {
        let data = self.get_profile(key).await?;
        fs::write(path, data).await?;
        Ok(())
    }

    /// Publishes a pre-uploaded package.
    ///
    /// The contents must already have been uploaded by calling [`Client::initiate_upload`],
    /// streaming the data to the returned URLs, and finally calling [`Client::finish_upload`].
    ///
    /// This method requires a valid API token on the client.
    pub async fn submit_package(
        &self,
        upload_uuid: Uuid,
        mut metadata: PackageMetadata,
    ) -> Result<PackageSubmissionResult> {
        let url = self.experimental_url("submission/submit");
        metadata.upload_uuid = Some(upload_uuid);

        let response = self
            .auth_request(Method::POST, url)?
            .json(&metadata)
            .send()
            .await
            .handle()?
            .json()
            .await?;

        Ok(response)
    }

    pub(crate) fn experimental_url(&self, tail: impl Display) -> String {
        format!("{}/api/experimental/{}/", self.base_url, tail)
    }
}
