use base64::{prelude::BASE64_STANDARD, Engine};
use bytes::Bytes;
use uuid::Uuid;

use crate::{models::*, prelude::*, util, Error, Result};

const PROFILE_DATA_PREFIX: &str = "#r2modman\n";

impl Client {
    /// Creates a profile with the given data and returns its key.
    ///
    /// The data is expected to be a ZIP archive containing a `export.r2x` file and
    /// eventual config files or directories. However, any arbitrary data is allowed.
    ///
    /// The returned key is used to retrieve the profile with [`Client::get_profile`].
    pub async fn create_profile(&self, data: impl AsRef<[u8]>) -> Result<Uuid> {
        let mut base64 = String::from(PROFILE_DATA_PREFIX);
        base64.push_str(&BASE64_STANDARD.encode(data));

        self.create_profile_raw(base64.into_bytes()).await
    }

    /// Creates a profile with the given data and returns its key.
    ///
    /// The data is expected to be a base64-encoded ZIP archive containing a `export.r2x`
    /// file and eventual config files or directories. However, any arbitrary data is allowed.
    ///
    /// The returned key is used to retrieve the profile with [`Client::get_profile`].
    pub async fn create_profile_raw(&self, data: Vec<u8>) -> Result<Uuid> {
        let url = self.url("/experimental/legacyprofile/create");
        let headers = util::header_map([("Content-Type", "application/octet-stream")]);

        let response = self
            .post(url, data, Some(headers))
            .await?
            .json::<LegacyProfileCreateResponse>()
            .await?;

        Ok(response.key)
    }

    /// Downloads a profile with the given key.
    ///
    /// The returned data is usually a ZIP archive containing a `export.r2x` file and eventual config files.
    ///
    /// This assumes the profile is encoded with base64, but any arbitrary data is allowed.
    pub async fn get_profile(&self, key: Uuid) -> Result<Vec<u8>> {
        let bytes = self.get_profile_raw(key).await?;
        let text = String::from_utf8_lossy(&bytes);

        match text.strip_prefix(PROFILE_DATA_PREFIX) {
            Some(base64) => BASE64_STANDARD.decode(base64).map_err(Error::Base64),
            None => Err(Error::InvalidProfileData),
        }
    }

    /// Downloads a profile with the given key.
    ///
    /// The returned data is usually a base64-encoded ZIP archive containing a `export.r2x`
    /// file and eventual config files, but any arbitrary data is allowed.
    pub async fn get_profile_raw(&self, key: Uuid) -> Result<Bytes> {
        let url = self.url(format_args!("/experimental/legacyprofile/get/{}", key));
        let response = self.get(url).await?.bytes().await?;
        Ok(response)
    }
}
