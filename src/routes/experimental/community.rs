use url::Url;

use crate::{models::*, prelude::*, Result};

impl Client {
    /// Fetches a page of communities.
    ///
    /// Returns a [`CursorState`] which is used to navigate between pages by passing the
    /// fields in the `cursor` parameter.
    pub async fn get_communities(
        &self,
        cursor: Option<impl AsRef<str>>,
    ) -> Result<(CursorState, Vec<Community>)> {
        let mut url = self.url("/experimental/community");
        if let Some(cursor) = cursor {
            url.push_str(&format!("?cursor={}", cursor.as_ref()));
        }
        let response: PaginatedResponse<Community> = self.get_json(url).await?;
        Ok((response.pagination.into(), response.results))
    }

    /// Fetches a page of categories from the given community.
    ///
    /// Returns a [`CursorState`] which is used to navigate between pages by passing the
    /// fields in the `cursor` parameter.
    pub async fn get_categories(
        &self,
        community: impl AsRef<str>,
        cursor: Option<impl AsRef<str>>,
    ) -> Result<(CursorState, Vec<CommunityCategory>)> {
        let mut url = self.url(format_args!(
            "/experimental/community/{}",
            community.as_ref()
        ));
        if let Some(cursor) = cursor {
            url.push_str(&format!("?cursor={}", cursor.as_ref()));
        }
        let response: PaginatedResponse<CommunityCategory> = self.get_json(url).await?;
        Ok((response.pagination.into(), response.results))
    }
}

/// Returned by paginated endpoints and used to navigate between pages.
#[derive(Debug, Clone)]
pub struct CursorState {
    pub next: Option<String>,
    pub prev: Option<String>,
}

impl From<Pagination> for CursorState {
    fn from(value: Pagination) -> Self {
        return CursorState {
            next: map_url(value.next_link),
            prev: map_url(value.previous_link),
        };

        fn map_url(url: Option<Url>) -> Option<String> {
            url.and_then(|url| {
                url.query_pairs()
                    .find(|(key, _)| key == "cursor")
                    .map(|(_, value)| value.into_owned())
            })
        }
    }
}
