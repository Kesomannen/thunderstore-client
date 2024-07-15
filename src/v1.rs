use crate::{models::*, Client, IntoPackageId, IntoVersionId, ResponseExt, Result};
use std::fmt::Display;

impl Client {
    /// Fetches [`PackageMetrics`] for a specific package.
    /// 
    /// `community` is the slug of the community, which is usually in kebab-case.
    pub async fn get_metrics(
        &self,
        community: impl Display,
        package: impl IntoPackageId,
    ) -> Result<PackageMetrics> {
        let url = self.v1_url(
            community,
            format_args!("package-metrics/{}", package.into_id()?.path()),
        );
        let response = self.client.get(&url).send().await.handle()?.json().await?;
        Ok(response)
    }

    /// Fetches the download count for a specific version of a package.
    /// 
    /// `community` is the slug of the community, which is usually in kebab-case.
    pub async fn get_downloads(
        &self,
        community: impl Display,
        version: impl IntoVersionId,
    ) -> Result<u64> {
        let url = self.v1_url(
            community,
            format_args!("package-metrics/{}", version.into_id()?.path()),
        );
        let response: PackageVersionMetrics =
            self.client.get(&url).send().await.handle()?.json().await?;
        Ok(response.downloads)
    }

    /// Fetches all available packages in a community, which is referred to by its slug.
    ///
    /// Note that on popular communities like Lethal Company (`lethal-company`),
    /// this will fetch up to 170 MB of data.
    /// 
    pub async fn list_packages_v1(&self, community: impl Display) -> Result<Vec<PackageListing>> {
        let url = self.v1_url(community, "package");
        let response = self.client.get(&url).send().await.handle()?.json().await?;
        Ok(response)
    }

    fn v1_url(&self, community: impl Display, tail: impl Display) -> String {
        format!("{}/c/{}/api/v1/{}/", self.base_url, community, tail)
    }
}
