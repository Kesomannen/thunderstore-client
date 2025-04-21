use crate::{
    models::*, usermedia::PackageMetadata, util, Client, Error, IntoPackageId, IntoVersionId,
    Result,
};
use base64::{prelude::BASE64_STANDARD, Engine};
use std::{fmt::Display, path::Path};
use tokio::fs;
use uuid::Uuid;

const PROFILE_DATA_PREFIX: &str = "#r2modman\n";

impl Client {
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
    pub async fn get_package(&self, id: impl IntoPackageId<'_>) -> Result<Package> {
        let url = self.experimental_url(format_args!("/package/{}", id.into_id()?.path()));
        self.get_json(url).await
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
    pub async fn get_version(&self, id: impl IntoVersionId<'_>) -> Result<PackageVersion> {
        let url = self.experimental_url(format_args!("/package/{}", id.into_id()?.path()));
        self.get_json(url).await
    }

    /// Fetches the changelog for a specific version of a package.
    /// The changelog is returned as a markdown string.
    ///
    /// Note that a package may not have a changelog, in which case [`Error::NotFound`] is returned.
    pub async fn get_changelog(&self, id: impl IntoVersionId<'_>) -> Result<String> {
        let url =
            self.experimental_url(format_args!("/package/{}/changelog", id.into_id()?.path()));
        let response: MarkdownResponse = self.get_json(url).await?;
        Ok(response.markdown)
    }

    /// Fetches the readme for a specific version of a package.
    /// The readme is returned as a markdown string.
    pub async fn get_readme(&self, id: impl IntoVersionId<'_>) -> Result<String> {
        let url = self.experimental_url(format_args!("/package/{}/readme", id.into_id()?.path()));
        let response: MarkdownResponse = self.get_json(url).await?;
        Ok(response.markdown)
    }

    /// Renders a markdown string to HTML.
    pub async fn render_markdown(&self, markdown: impl ToString) -> Result<String> {
        let url = self.experimental_url("/frontend/render-markdown");
        let response: RenderMarkdownResponse = self
            .post_json(
                url,
                &RenderMarkdownParams {
                    markdown: markdown.to_string(),
                },
            )
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

        let url = self.experimental_url("/legacyprofile/create");
        let headers = util::header_map([("Content-Type", "application/octet-stream")]);

        let response = self
            .post(url, base64, Some(headers))
            .await?
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
        let url = self.experimental_url(format_args!("/legacyprofile/get/{}", key));

        let response = self.get(url).await?.text().await?;

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
        let url = self.experimental_url("/submission/submit");
        metadata.upload_uuid = Some(upload_uuid);

        let response = self.post_json(url, &metadata).await?.json().await?;
        Ok(response)
    }

    pub(crate) fn experimental_url(&self, path: impl Display) -> String {
        format!("{}/api/experimental{}", self.base_url, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{pin_mut, TryStreamExt};

    #[tokio::test]
    async fn stream_packages_v1() -> Result<()> {
        let client = Client::new();

        let stream = client.stream_packages_v1("muck").await?;
        pin_mut!(stream);

        let mut count = 0;
        while let Some(_) = stream.try_next().await? {
            count += 1;
        }

        assert!(count > 0);

        Ok(())
    }

    #[tokio::test]
    async fn get_package() -> Result<()> {
        let client = Client::new();

        client.get_package(("Kesomannen", "GaleModManager")).await?;
        client.get_package("Kesomannen-GaleModManager").await?;

        Ok(())
    }

    #[tokio::test]
    async fn get_package_fails_when_not_found() -> Result<()> {
        let client = Client::new();

        match client.get_package(("Kesomannen", "GaleModManager2")).await {
            Err(Error::NotFound) => (),
            other => panic!("expected NotFound error, got {:?}", other),
        }

        Ok(())
    }

    #[tokio::test]
    async fn get_version() -> Result<()> {
        let client = Client::new();

        client
            .get_version(("Kesomannen", "GaleModManager", "0.6.0"))
            .await?;
        client
            .get_version("Kesomannen-GaleModManager-0.6.0")
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn get_changelog() -> Result<()> {
        Client::new()
            .get_changelog(("Kesomannen", "GaleModManager", "0.1.0"))
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn get_readme() -> Result<()> {
        Client::new()
            .get_readme(("Kesomannen", "GaleModManager", "0.1.0"))
            .await?;
        Ok(())
    }
}
