use std::{
    borrow::Cow,
    cmp,
    fmt::{self, Debug, Display},
    hash::Hash,
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

use super::package::PackageIdent;

/// A unique identifier for a package version, formatted as `namespace-name-version`
/// and also known as a dependency string.
///
/// This struct is flexible and can be created in a number of ways:
/// ```
/// use thunderstore::VersionIdent;
///
/// let a = VersionIdent::new("BepInEx", "BepInExPack", "5.4.2100");
/// let b: VersionIdent = "BepInEx-BepInExPack-5.4.2100".parse().unwrap();
/// let c: VersionIdent = ("BepInEx", "BepInExPack", "5.4.2100").into();
/// let d = ("BepInEx", "BepInExPack", &semver::Version::new(5, 4, 2100)).into();
/// ```
///
/// Most methods on [`crate::Client`] accept any type that implements [`IntoVersionIdent`],
/// which allows any of the above methods to be used interchangeably.
#[derive(Eq, Clone, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct VersionIdent {
    pub(super) repr: Cow<'static, str>,
    pub(super) name_start: usize,
    pub(super) version_start: usize,
}

impl VersionIdent {
    pub fn new(
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
        version: impl AsRef<str>,
    ) -> Self {
        let namespace = namespace.as_ref();
        let name = name.as_ref();

        let repr = Cow::Owned(format!("{}-{}-{}", namespace, name, version.as_ref()));
        let name_start = namespace.len() + 1;
        let version_start = name_start + name.len() + 1;

        Self {
            repr,
            name_start,
            version_start,
        }
    }

    #[inline]
    pub fn namespace(&self) -> &str {
        &self.repr[..self.name_start - 1]
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.repr[self.name_start..self.version_start - 1]
    }

    #[inline]
    pub fn version(&self) -> &str {
        &self.repr[self.version_start..]
    }

    pub fn parsed_version(&self) -> semver::Version {
        self.version()
            .parse()
            .expect("invalid version in VersionIdent")
    }

    /// Returns an object that, when formatted with `{}`, will produce the URL path for this version.
    ///
    /// ## Example
    ///
    /// ```
    /// let id = thunderstore::VersionId::new("BepInEx", "BepInExPack", "5.4.2100");
    /// assert_eq!(id.path().to_string(), "BepInEx/BepInExPack/5.4.2100");
    /// ```
    #[inline]
    pub fn path(&self) -> impl Display + '_ {
        VersionIdPath::new(self)
    }

    #[inline]
    pub fn into_cow(self) -> Cow<'static, str> {
        self.repr
    }

    /// Consumes the [`VersionId`] and returns the underlying string, formatted as `namespace-name-version`.
    #[inline]
    pub fn into_string(self) -> String {
        self.repr.into_owned()
    }

    /// Returns a reference to the underlying string, formatted as `namespace-name-version`.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.repr
    }

    pub fn package_id(&self) -> PackageIdent {
        let repr = Cow::Owned(self.repr[..self.version_start - 1].to_string());

        PackageIdent {
            repr,
            name_start: self.name_start,
        }
    }
}

impl PartialEq for VersionIdent {
    fn eq(&self, other: &Self) -> bool {
        self.repr == other.repr
    }
}

impl Ord for VersionIdent {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.repr.cmp(&other.repr)
    }
}

impl PartialOrd for VersionIdent {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for VersionIdent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.repr.hash(state);
    }
}

impl AsRef<str> for VersionIdent {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<VersionIdent> for Cow<'static, str> {
    fn from(id: VersionIdent) -> Self {
        id.into_cow()
    }
}

impl From<VersionIdent> for String {
    fn from(id: VersionIdent) -> Self {
        id.into_string()
    }
}

impl Display for VersionIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl Debug for VersionIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VersionId").field(&self.repr).finish()
    }
}

impl TryFrom<Cow<'static, str>> for VersionIdent {
    type Error = Error;

    fn try_from(value: Cow<'static, str>) -> Result<Self> {
        let mut indices = value.match_indices('-').map(|(i, _)| i);

        let name_start = indices.next().ok_or(Error::InvalidPackageId)? + 1;
        let version_start = indices.next().ok_or(Error::InvalidPackageId)? + 1;

        Ok(Self {
            repr: value,
            name_start,
            version_start,
        })
    }
}

impl TryFrom<String> for VersionIdent {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        VersionIdent::try_from(Cow::Owned(value))
    }
}

impl TryFrom<&'static str> for VersionIdent {
    type Error = Error;

    fn try_from(value: &'static str) -> Result<Self> {
        VersionIdent::try_from(Cow::Borrowed(value))
    }
}

impl FromStr for VersionIdent {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        s.to_string().try_into()
    }
}

impl<T, U, V> From<(T, U, V)> for VersionIdent
where
    T: AsRef<str>,
    U: AsRef<str>,
    V: AsRef<str>,
{
    fn from((namespace, name, version): (T, U, V)) -> Self {
        Self::new(namespace, name, version)
    }
}

struct VersionIdPath<'a> {
    id: &'a VersionIdent,
}

impl<'a> VersionIdPath<'a> {
    pub fn new(id: &'a VersionIdent) -> Self {
        Self { id }
    }
}

impl<'a> Display for VersionIdPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}",
            self.id.namespace(),
            self.id.name(),
            self.id.version()
        )
    }
}

pub trait IntoVersionIdent<'a> {
    fn into_id(self) -> Result<Cow<'a, VersionIdent>>;
}

impl<T> IntoVersionIdent<'_> for T
where
    T: Into<VersionIdent>,
{
    fn into_id(self) -> Result<Cow<'static, VersionIdent>> {
        Ok(Cow::Owned(self.into()))
    }
}

impl<'a> IntoVersionIdent<'a> for String {
    fn into_id(self) -> Result<Cow<'static, VersionIdent>> {
        self.try_into().map(Cow::Owned)
    }
}

impl IntoVersionIdent<'_> for &str {
    fn into_id(self) -> Result<Cow<'static, VersionIdent>> {
        self.parse().map(Cow::Owned)
    }
}

impl<'a> IntoVersionIdent<'a> for &'a VersionIdent {
    fn into_id(self) -> Result<Cow<'a, VersionIdent>> {
        Ok(Cow::Borrowed(self))
    }
}
