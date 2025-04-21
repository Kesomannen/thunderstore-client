//! A library for interacting with the Thunderstore API.
//!
//! The main struct is [`Client`], which provides methods for fetching, downloading and publishing packages.
//! The easiest way to get up and running is to use the [`Client::new`] method, which creates a client with the default configuration.
//! If you need more control over the client's configuration, use the [`Client::builder`] method instead (see [`ClientBuilder`]).
//!
//! Some methods, including uploading and submitting packages, require an API token to be set on the client.
//! You can set this token using the [`ClientBuilder::with_token`] method.
//!
//! # Examples
//!
//! ```no_run
//! #[tokio::main]
//! async fn main() -> thunderstore::Result<()> {
//!     let client = thunderstore::Client::builder()
//!         .with_token("tss_XXX")
//!         .build()?;
//!
//!     let package = client.get_package(("Kesomannen", "GaleModManager")).await?;
//!     client.download_to_dir(&package.latest, r"C:\Users\bobbo\Downloads").await?;
//!
//!     client.publish_file(
//!         "CoolMod.zip",
//!         PackageMetadata::new("Kesomannen", ["lethal-company"])
//!             .with_global_categories(["tools"])
//!             .with_categories("lethal-company", ["serverside"])
//!     ).await?;
//! }
//! ```

mod client;
mod error;
mod id;
mod routes;
mod util;

pub mod models;

pub use client::{Client, ClientBuilder};
pub use error::{Error, Result};
pub use id::{IntoPackageId, IntoVersionId, PackageId, VersionId};
pub use routes::*;
