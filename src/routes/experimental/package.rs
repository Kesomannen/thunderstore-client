use crate::{models::*, prelude::*};

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
    pub async fn get_package(&self, id: impl IntoPackageIdent<'_>) -> Result<Package> {
        let url = self.url(format_args!(
            "/experimental/package/{}",
            id.into_id()?.path()
        ));
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
    pub async fn get_version(&self, id: impl IntoVersionIdent<'_>) -> Result<PackageVersion> {
        let url = self.url(format_args!(
            "/experimental/package/{}",
            id.into_id()?.path()
        ));
        self.get_json(url).await
    }

    /// Fetches the readme for a specific version of a package.
    /// The readme is returned as a markdown string.
    pub async fn get_readme(&self, id: impl IntoVersionIdent<'_>) -> Result<String> {
        let url = self.url(format_args!(
            "/experimental/package/{}/readme",
            id.into_id()?.path()
        ));
        let response: MarkdownResponse = self.get_json(url).await?;
        Ok(response.markdown)
    }

    /// Fetches the changelog for a specific version of a package.
    /// The changelog is returned as a markdown string.
    ///
    /// Note that a package may not have a changelog, in which case [`Error::NotFound`] is returned.
    pub async fn get_changelog(&self, id: impl IntoVersionIdent<'_>) -> Result<String> {
        let url = self.url(format_args!(
            "/experimental/package/{}/changelog",
            id.into_id()?.path()
        ));
        let response: MarkdownResponse = self.get_json(url).await?;
        Ok(response.markdown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{pin_mut, TryStreamExt};

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
