use base64::{prelude::BASE64_STANDARD, Engine};
use uuid::Uuid;

use crate::{models::*, prelude::*, util};

const PROFILE_DATA_PREFIX: &str = "#r2modman\n";

impl Client {
    /// Creates a profile with the given data and returns its key.
    ///
    /// The data is expected to be a ZIP archive containing a `mods.yml` file and
    /// any addition configuration files or directories. However, any arbitrary
    /// data is allowed, but will likely fail to import correctly in mod managers.
    ///
    /// The key returned is used to retrieve the profile with [`Client::get_profile`].
    pub async fn create_profile(&self, data: impl AsRef<[u8]>) -> Result<Uuid> {
        let mut base64 = String::from(PROFILE_DATA_PREFIX);
        base64.push_str(&BASE64_STANDARD.encode(data));

        let url = self.url("/experimental/legacyprofile/create");
        let headers = util::header_map([("Content-Type", "application/octet-stream")]);

        let response = self
            .post(url, base64, Some(headers))
            .await?
            .json::<LegacyProfileCreateResponse>()
            .await?;

        Ok(response.key)
    }

    /// Downloads a profile with the given key.
    ///
    /// The returned data is usually a ZIP archive containing a mods.yml file and
    /// any additional configuration files. However, any arbitrary data is allowed
    /// by Thunderstore.
    pub async fn get_profile(&self, key: Uuid) -> Result<Vec<u8>> {
        let url = self.url(format_args!("/experimental/legacyprofile/get/{}", key));

        let response = self.get(url).await?.text().await?;

        match response.strip_prefix(PROFILE_DATA_PREFIX) {
            Some(data) => BASE64_STANDARD.decode(data).map_err(Error::Base64),
            None => Err(Error::InvalidProfileData),
        }
    }
}
