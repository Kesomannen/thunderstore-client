use reqwest::Method;

use crate::{models::*, prelude::*, Result};

impl Client {
    /// Fetches an index of all the package wikis on Thunderstore.
    pub async fn get_wikis(&self) -> Result<WikisResponse> {
        self.get_json(self.url("/experimental/package/wikis")).await
    }

    /// Fetches the wiki of a specific package.
    pub async fn get_wiki(&self, package: impl IntoPackageIdent<'_>) -> Result<Wiki> {
        self.get_json(self.wiki_url(package)?).await
    }

    /// Creates a package wiki page.
    pub async fn create_wiki_page(
        &self,
        package: impl IntoPackageIdent<'_>,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<WikiPage> {
        self.upsert_wiki_page(
            package,
            WikiPageUpsert {
                id: None,
                title: title.into(),
                content: content.into(),
            },
        )
        .await
    }

    /// Updates a package wiki page.
    pub async fn update_wiki_page(
        &self,
        package: impl IntoPackageIdent<'_>,
        id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<WikiPage> {
        self.upsert_wiki_page(
            package,
            WikiPageUpsert {
                id: Some(id.into()),
                title: title.into(),
                content: content.into(),
            },
        )
        .await
    }

    /// Upserts a package wiki page.
    ///
    /// If `upsert.id` is `None`, this creates a new page.
    /// Otherwise, an existing one is updated according to its `id`.
    pub async fn upsert_wiki_page(
        &self,
        package: impl IntoPackageIdent<'_>,
        upsert: WikiPageUpsert,
    ) -> Result<WikiPage> {
        let page = self
            .post_json(self.wiki_url(package)?, &upsert)
            .await?
            .json()
            .await?;
        Ok(page)
    }

    /// Deletes a package wiki page.
    pub async fn delete_wiki_page(
        &self,
        package: impl IntoPackageIdent<'_>,
        id: impl Into<String>,
    ) -> Result<()> {
        let body = serde_json::to_string(&WikiPageDelete { id: id.into() })?;
        self.request(
            Method::DELETE,
            self.wiki_url(package)?,
            Some(body.into()),
            None,
        )
        .await?;
        Ok(())
    }

    /// Fetches a package wiki page by its `id`.
    pub async fn get_wiki_page(&self, id: impl AsRef<str>) -> Result<WikiPage> {
        let url = self.url(format_args!("/experimental/wiki/page/{}", id.as_ref()));
        self.get_json(url).await
    }

    fn wiki_url<'a>(&self, package: impl IntoPackageIdent<'a>) -> Result<String> {
        Ok(self.url(format_args!(
            "/experimental/package/{}/wiki",
            package.into_id()?.path()
        )))
    }
}
