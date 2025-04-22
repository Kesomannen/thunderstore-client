use crate::{models::*, prelude::*, Result};

impl Client {
    /// Renders a markdown string to HTML.
    pub async fn render_markdown(&self, markdown: impl Into<String>) -> Result<String> {
        let url = self.url("/experimental/frontend/render-markdown");
        let response: RenderMarkdownResponse = self
            .post_json(
                url,
                &RenderMarkdownParams {
                    markdown: markdown.into(),
                },
            )
            .await?
            .json::<RenderMarkdownResponse>()
            .await?;

        Ok(response.html)
    }
}
