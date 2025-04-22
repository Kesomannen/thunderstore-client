use crate::{models::*, prelude::*};

impl Client {
    pub async fn get_current_community(&self) -> Result<Community> {
        self.get_json(self.url("/experimental/current-community"))
            .await
    }
}
