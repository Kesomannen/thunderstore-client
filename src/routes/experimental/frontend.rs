use crate::{models::*, prelude::*};

impl Client {
    /// Renders a markdown string to HTML.
    pub async fn render_markdown(&self, markdown: impl ToString) -> Result<String> {
        let url = self.url("/experimental/frontend/render-markdown");
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
}
