use crate::{PackageIdent, VersionIdent};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, hash::Hash};
use url::Url;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
#[non_exhaustive]
pub struct PackageV1 {
    #[serde(rename = "uuid4")]
    pub uuid: Uuid,
    #[serde(rename = "owner")]
    pub namespace: String,
    pub name: String,
    #[serde(rename = "full_name")]
    pub ident: PackageIdent,
    pub categories: HashSet<String>,
    pub date_created: DateTime<Utc>,
    pub date_updated: DateTime<Utc>,
    pub donation_link: Option<Url>,
    pub has_nsfw_content: bool,
    pub is_deprecated: bool,
    pub is_pinned: bool,
    pub package_url: Url,
    pub rating_score: u32,
    pub versions: Vec<PackageVersionV1>,
}

impl PackageV1 {
    pub fn latest(&self) -> &PackageVersionV1 {
        &self.versions[0]
    }

    pub fn is_modpack(&self) -> bool {
        self.categories.contains("Modpacks")
    }

    pub fn version_by_id(&self, uuid: &Uuid) -> Option<&PackageVersionV1> {
        self.versions.iter().find(|v| v.uuid == *uuid)
    }

    pub fn version_by_name(&self, version: &semver::Version) -> Option<&PackageVersionV1> {
        self.versions.iter().find(|v| v.number == *version)
    }

    pub fn total_downloads(&self) -> u32 {
        self.versions.iter().map(|v| v.downloads).sum()
    }
}

impl Hash for PackageV1 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

impl PartialEq for PackageV1 {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
#[non_exhaustive]
pub struct PackageVersionV1 {
    #[serde(rename = "uuid4")]
    pub uuid: Uuid,
    pub name: String,
    #[serde(rename = "version_number")]
    pub number: semver::Version,
    #[serde(rename = "full_name")]
    pub ident: VersionIdent,
    pub date_created: DateTime<Utc>,
    pub dependencies: Vec<VersionIdent>,
    pub description: String,
    pub download_url: Url,
    pub downloads: u32,
    pub file_size: u64,
    pub icon: Url,
    pub is_active: bool,
    pub website_url: String,
}

impl PartialEq for PackageVersionV1 {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Hash for PackageVersionV1 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct LegacyProfileCreateResponse {
    pub key: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct UserMediaInitiateUploadParams {
    #[serde(rename = "filename")]
    pub name: String,
    #[serde(rename = "file_size_bytes")]
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct UserMediaInitiateUploadResponse {
    pub user_media: UserMedia,
    pub upload_urls: Vec<UploadPartUrl>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct UserMedia {
    pub uuid: Uuid,
    #[serde(rename = "filename")]
    pub name: String,
    pub size: u64,
    #[serde(rename = "datetime_created")]
    pub date_created: DateTime<Utc>,
    pub expiry: DateTime<Utc>,
    pub status: UserMediaStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum UserMediaStatus {
    Initial,
    UploadInitiated,
    UploadCreated,
    UploadError,
    UploadComplete,
    UploadAborted,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct UploadPartUrl {
    #[serde(rename = "part_number")]
    pub number: u32,
    pub url: String,
    pub offset: u64,
    pub length: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct UserMediaFinishUploadParams {
    pub parts: Vec<CompletedPart>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompletedPart {
    #[serde(rename = "ETag")]
    pub tag: String,
    #[serde(rename = "PartNumber")]
    pub number: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct PackageSubmissionResult {
    pub package_version: PackageVersion,
    pub available_communities: Vec<AvailableCommunity>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct AvailableCommunity {
    pub community: Community,
    pub categories: CommunityCategory,
    pub url: Url,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct CommunityCategory {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct PackageVersionMetrics {
    pub downloads: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct PackageMetrics {
    pub downloads: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct RenderMarkdownParams {
    pub markdown: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct RenderMarkdownResponse {
    pub html: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct MarkdownResponse {
    pub markdown: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct PackageIndexEntry {
    pub namespace: String,
    pub name: String,
    pub version_number: semver::Version,
    pub file_format: Option<String>,
    pub file_size: u64,
    pub dependencies: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
#[non_exhaustive]
pub struct PackageVersion {
    #[serde(rename = "full_name")]
    pub ident: VersionIdent,
    pub description: String,
    pub icon: Url,
    pub dependencies: Vec<VersionIdent>,
    pub download_url: Url,
    pub downloads: u32,
    pub date_created: DateTime<Utc>,
    pub website_url: String,
    pub is_active: bool,
}

impl PartialEq for PackageVersion {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident
    }
}

impl Hash for PackageVersion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ident.hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
#[non_exhaustive]
pub struct Package {
    #[serde(rename = "full_name")]
    pub ident: PackageIdent,
    pub package_url: Url,
    pub date_created: DateTime<Utc>,
    pub date_updated: DateTime<Utc>,
    pub rating_score: i32,
    pub is_pinned: bool,
    pub is_deprecated: bool,
    pub total_downloads: i32,
    pub latest: PackageVersion,
    pub community_listings: Vec<PackageListingExperimental>,
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident
    }
}

impl Hash for Package {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ident.hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct PackageListingExperimental {
    pub has_nsfw_content: bool,
    pub categories: HashSet<String>,
    pub community: String,
    pub review_status: ReviewStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ReviewStatus {
    Unreviewed,
    Approved,
    Rejected,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct PaginatedResponse<T> {
    pub pagination: Pagination,
    pub results: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Pagination {
    pub next_link: Option<Url>,
    pub previous_link: Option<Url>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Community {
    #[serde(rename = "identifier")]
    pub ident: String,
    pub name: String,
    pub discord_url: Option<Url>,
    pub wiki_url: Option<Url>,
    pub require_package_listing_approval: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct IconValidatorParams {
    pub icon_data: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ManifestV1ValidatorParams {
    pub namespace: String,
    pub manifest_data: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ReadmeValidatorParams {
    pub readme_data: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ValidatorResponse {
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WikisResponse {
    pub results: Vec<ListedWiki>,
    pub cursor: DateTime<Utc>,
    pub has_more: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ListedWiki {
    pub namespace: String,
    pub name: String,
    pub wiki: Wiki,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Wiki {
    pub id: String,
    pub title: String,
    pub slug: String,
    #[serde(rename = "datetime_created")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "datetime_updated")]
    pub updated_at: DateTime<Utc>,
    pub pages: Vec<WikiPage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WikiPage {
    pub id: String,
    pub title: String,
    pub slug: String,
    #[serde(rename = "datetime_created")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "datetime_updated")]
    pub updated_at: DateTime<Utc>,
    #[serde(default, rename = "markdown_content")]
    pub content: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WikiPageUpsert {
    pub id: Option<String>,
    pub title: String,
    #[serde(rename = "markdown_content")]
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WikiPageDelete {
    pub id: String,
}
