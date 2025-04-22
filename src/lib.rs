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
//! use
//!
//! #[tokio::main]
//! async fn main() -> thunderstore::Result<()> {
//!     let client = thunderstore::Client::builder()
//!         .with_token("tss_XXX")
//!         .build()?;
//!
//!     let package = client.get_package(("Kesomannen", "GaleModManager")).await?;
//!     let _bytes = client.download(package.latest.ident).await?;
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
mod ident;
mod routes;
mod util;

pub mod models;

pub use client::{Client, ClientBuilder};
pub use error::{Error, Result};
pub use ident::{IntoPackageIdent, IntoVersionIdent, PackageIdent, VersionIdent};
pub use routes::*;

pub mod prelude {
    pub use crate::{
        models::{Package, PackageV1, PackageVersion, PackageVersionV1},
        Client, ClientBuilder, IntoPackageIdent, IntoVersionIdent, PackageIdent, VersionIdent,
    };
}
