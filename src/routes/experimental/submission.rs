use std::fmt::Display;

use base64::{prelude::BASE64_STANDARD, Engine};
use serde::Serialize;
use uuid::Uuid;

use crate::{models::*, prelude::*, Result};

use super::usermedia::PackageMetadata;

impl Client {
    /// Publishes a pre-uploaded package.
    ///
    /// The contents must already have been uploaded by calling [`Client::initiate_upload`],
    /// streaming the data to the returned URLs, and finally calling [`Client::finish_upload`].
    ///
    /// This method requires an API token on the client.
    pub async fn submit_package(
        &self,
        upload_uuid: Uuid,
        mut metadata: PackageMetadata,
    ) -> Result<PackageSubmissionResult> {
        let url = self.url("/experimental/submission/submit");
        metadata.upload_uuid = Some(upload_uuid);

        let response = self.post_json(url, &metadata).await?.json().await?;
        Ok(response)
    }

    /// Validates a package icon.
    pub async fn validate_icon(&self, data: impl AsRef<[u8]>) -> Result<bool> {
        let icon_data = BASE64_STANDARD.encode(data);
        self.validate("/icon", IconValidatorParams { icon_data })
            .await
    }

    /// Validates a package manifest (v1) as if uploaded in given namespace.
    pub async fn validate_manifest_v1(
        &self,
        namespace: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<bool> {
        self.validate(
            "/manifest-v1",
            ManifestV1ValidatorParams {
                namespace: namespace.into(),
                manifest_data: content.into(),
            },
        )
        .await
    }

    /// Validates a package README.
    pub async fn validate_readme(&self, content: impl Into<String>) -> Result<bool> {
        self.validate(
            "/readme",
            ReadmeValidatorParams {
                readme_data: content.into(),
            },
        )
        .await
    }

    async fn validate(&self, path: impl Display, params: impl Serialize) -> Result<bool> {
        let url = self.url(format_args!("/experimental/submission/validate{path}"));
        let response: ValidatorResponse = self.post_json(url, &params).await?.json().await?;
        Ok(response.success)
    }
}
