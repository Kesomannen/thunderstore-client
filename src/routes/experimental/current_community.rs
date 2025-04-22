use crate::{models::*, prelude::*, Result};

impl Client {
    /// Fetches the start/main community of the client's base URL.
    ///
    /// For the default thunderstore repo, this is `riskofrain2`.
    pub async fn get_current_community(&self) -> Result<Community> {
        self.get_json(self.url("/experimental/current-community"))
            .await
    }
}
