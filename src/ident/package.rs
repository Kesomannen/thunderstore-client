use std::{
    borrow::Cow,
    cmp,
    fmt::{self, Debug, Display},
    hash::Hash,
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

use super::VersionIdent;

/// A unique identifier for a package, formatted as `namespace-name`.
///
/// This struct can be created in a number of ways:
/// ```
/// use thunderstore::PackageId;
///
/// let a = PackageId::new("BepInEx", "BepInExPack");
/// let b: PackageId = "BepInEx-BepInExPack".parse().unwrap();
/// let c: PackageId = ("BepInEx", "BepInExPack").into();
/// ```
///
/// Most methods on [`crate::Client`] accept any type that implements [`IntoPackageIdent`],
/// which allows any of the above methods to be used interchangeably.
#[derive(Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PackageIdent {
    pub(super) repr: Cow<'static, str>,
    pub(super) name_start: usize,
}

impl PackageIdent {
    pub fn new(namespace: impl AsRef<str>, name: impl AsRef<str>) -> Self {
        let namespace = namespace.as_ref();

        let repr = Cow::Owned(format!("{}-{}", namespace, name.as_ref()));
        let name_start = namespace.len() + 1;

        Self { repr, name_start }
    }

    #[inline]
    pub fn namespace(&self) -> &str {
        &self.repr[..self.name_start - 1]
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.repr[self.name_start..]
    }

    /// Returns an object that, when formatted with `{}`, will produce the URL path for this package.
    ///
    /// ## Example
    ///
    /// ```
    /// let id = thunderstore::PackageId::new("BepInEx", "BepInExPack");
    /// assert_eq!(id.path().to_string(), "BepInEx/BepInExPack");
    /// ```
    #[inline]
    pub fn path(&self) -> impl Display + '_ {
        PackageIdPath::new(self)
    }

    #[inline]
    pub fn into_cow(self) -> Cow<'static, str> {
        self.repr
    }

    /// Consumes the [`PackageId`] and returns the underlying string, formatted as `namespace-name`.
    #[inline]
    pub fn into_string(self) -> String {
        self.repr.into_owned()
    }

    /// Returns a reference to the underlying string, formatted as `namespace-name`.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.repr
    }

    pub fn with_version(&self, version: impl Display) -> VersionIdent {
        let repr = Cow::Owned(format!("{}-{}", self.repr, version));
        let version_start = self.repr.len() + 1;

        VersionIdent {
            repr,
            name_start: self.name_start,
            version_start,
        }
    }
}

impl PartialEq for PackageIdent {
    fn eq(&self, other: &Self) -> bool {
        self.repr == other.repr
    }
}

impl Ord for PackageIdent {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.repr.cmp(&other.repr)
    }
}

impl PartialOrd for PackageIdent {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for PackageIdent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.repr.hash(state);
    }
}

impl AsRef<str> for PackageIdent {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<PackageIdent> for Cow<'static, str> {
    fn from(id: PackageIdent) -> Self {
        id.into_cow()
    }
}

impl From<PackageIdent> for String {
    fn from(id: PackageIdent) -> Self {
        id.into_string()
    }
}

impl Display for PackageIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl Debug for PackageIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PackageId").field(&self.repr).finish()
    }
}

impl TryFrom<Cow<'static, str>> for PackageIdent {
    type Error = Error;

    fn try_from(value: Cow<'static, str>) -> Result<Self> {
        let mut indices = value.match_indices('-').map(|(i, _)| i);

        let name_start = indices.next().ok_or(Error::InvalidPackageId)? + 1;

        Ok(Self {
            repr: value,
            name_start,
        })
    }
}

impl TryFrom<String> for PackageIdent {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        PackageIdent::try_from(Cow::Owned(value))
    }
}

impl TryFrom<&'static str> for PackageIdent {
    type Error = Error;

    fn try_from(value: &'static str) -> Result<Self> {
        PackageIdent::try_from(Cow::Borrowed(value))
    }
}

impl FromStr for PackageIdent {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        s.to_string().try_into()
    }
}

impl<T, U> From<(T, U)> for PackageIdent
where
    T: AsRef<str>,
    U: AsRef<str>,
{
    fn from((namespace, name): (T, U)) -> Self {
        Self::new(namespace, name)
    }
}

struct PackageIdPath<'a> {
    id: &'a PackageIdent,
}

impl<'a> PackageIdPath<'a> {
    pub fn new(id: &'a PackageIdent) -> Self {
        Self { id }
    }
}

impl<'a> Display for PackageIdPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.id.namespace(), self.id.name(),)
    }
}

impl From<&VersionIdent> for PackageIdent {
    fn from(id: &VersionIdent) -> Self {
        id.package_id()
    }
}

pub trait IntoPackageIdent<'a> {
    fn into_id(self) -> Result<Cow<'a, PackageIdent>>;
}

impl<T> IntoPackageIdent<'_> for T
where
    T: Into<PackageIdent>,
{
    fn into_id(self) -> Result<Cow<'static, PackageIdent>> {
        Ok(Cow::Owned(self.into()))
    }
}

impl IntoPackageIdent<'_> for String {
    fn into_id(self) -> Result<Cow<'static, PackageIdent>> {
        self.try_into().map(Cow::Owned)
    }
}

impl IntoPackageIdent<'_> for &str {
    fn into_id(self) -> Result<Cow<'static, PackageIdent>> {
        self.parse().map(Cow::Owned)
    }
}

impl<'a> IntoPackageIdent<'a> for &'a PackageIdent {
    fn into_id(self) -> Result<Cow<'a, PackageIdent>> {
        Ok(Cow::Borrowed(self))
    }
}
