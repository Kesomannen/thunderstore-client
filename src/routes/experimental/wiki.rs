use reqwest::Method;

use crate::{models::*, prelude::*};

impl Client {
    pub async fn get_wikis(&self) -> Result<WikisResponse> {
        self.get_json(self.url("/experimental/package/wikis")).await
    }

    pub async fn get_wiki(&self, package: impl IntoPackageIdent<'_>) -> Result<Wiki> {
        self.get_json(self.wiki_url(package)?).await
    }

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
