use crate::{models::*, Client, IntoPackageId, IntoVersionId, Result};
use async_stream::try_stream;
use futures_core::Stream;
use std::fmt::Display;

impl Client {
    /// Fetches [`PackageMetrics`] for a specific package.
    ///
    /// `community` is the slug of the community, which is usually in kebab-case.
    pub async fn get_metrics(
        &self,
        community: impl Display,
        package: impl IntoPackageId<'_>,
    ) -> Result<PackageMetrics> {
        let url = self.v1_url(
            community,
            format_args!("/package-metrics/{}", package.into_id()?.path()),
        );
        self.get_json(url).await
    }

    /// Fetches the download count for a specific version of a package.
    ///
    /// `community` is the slug of the community, which is usually in kebab-case.
    pub async fn get_downloads(
        &self,
        community: impl Display,
        version: impl IntoVersionId<'_>,
    ) -> Result<u64> {
        let url = self.v1_url(
            community,
            format_args!("/package-metrics/{}", version.into_id()?.path()),
        );
        let response: PackageVersionMetrics = self.get_json(url).await?;
        Ok(response.downloads)
    }

    /// Fetches all available packages in a community and collects them in a `Vec`.
    ///
    /// - `community` is the slug of the community, which is usually in kebab-case.
    ///
    /// Note that on popular communities like Lethal Company (`lethal-company`),
    /// this will fetch up to 170 MB of data.
    pub async fn list_packages_v1(&self, community: impl Display) -> Result<Vec<PackageV1>> {
        let url = self.v1_url(community, "/package");
        self.get_json(url).await
    }

    fn v1_url(&self, community: impl Display, path: impl Display) -> String {
        format!("{}/c/{}/api/v1{}", self.base_url, community, path)
    }
}

impl Client {
    /// Asynchronously streams all available packages in a community.
    ///
    /// - `community` is the slug of the community, which is usually in kebab-case.
    ///
    /// If you just want a `Vec` of all packages, use [`Client::list_packages_v1`] instead.
    ///
    /// ## Examples
    ///
    /// ```
    /// // provides `try_next`
    /// use futures_util::TryStreamExt;
    /// use futures_util::pin_mut;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = thunderstore::Client::new();
    ///
    ///     let stream = client.stream_packages_v1("content-warning").await?;
    ///     pin_mut!(stream); // needed for iteration
    ///
    ///     while let Some(package) = stream.try_next().await? {
    ///         println!("got {}!", package.name);
    ///     }
    ///
    ///    Ok(())
    /// }
    /// ```
    pub async fn stream_packages_v1(
        &self,
        community: impl Display,
    ) -> Result<impl Stream<Item = Result<PackageV1>>> {
        let url = self.v1_url(community, "/package");
        let mut response = self.get(url).await?;

        Ok(try_stream! {
            let mut buffer = Vec::new();
            let mut string = String::new();

            let mut is_first = true;

            while let Some(chunk) = response.chunk().await? {
                buffer.extend_from_slice(&chunk);

                let chunk = match std::str::from_utf8(&buffer) {
                    Ok(chunk) => chunk,
                    Err(_) => continue,
                };

                if is_first {
                    is_first = false;
                    string.extend(chunk.chars().skip(1)); // remove leading [
                } else {
                    string.push_str(chunk);
                }

                buffer.clear();

                while let Some(index) = string.find("}]},") {
                    let (json, _) = string.split_at(index + 3);
                    yield serde_json::from_str::<PackageV1>(json)?;
                    string.replace_range(..index + 4, "");
                }
            }
        })
    }
}
