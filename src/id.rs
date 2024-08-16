use std::{
    cmp,
    fmt::{self, Debug, Display},
    hash::Hash,
    str::FromStr,
};

use crate::{
    models::{Package, PackageVersion},
    Error, Result,
};

/// A unique identifier for a package version, often formatted as `namespace-name-version`
/// and also known as a dependency string.
///
/// This struct is flexible and can be created in a number of ways:
/// ```
/// use thunderstore::VersionId;
///
/// let a = VersionId::new("BepInEx", "BepInExPack", "5.4.2100");
/// let b = "BepInEx-BepInExPack-5.4.2100".parse().unwrap();
/// let c = ("BepInEx", "BepInExPack", "5.4.2100").into();
/// let d = ("BepInEx", "BepInExPack", &semver::Version::new(5, 4, 2100)).into();
/// ```
///
/// Most methods on [`Client`] accept any type that implements [`IntoVersionId`],
/// which allows any of the above methods to be used interchangeably.
#[derive(Eq, Clone)]
pub struct VersionId {
    repr: String,
    name_start: usize,
    version_start: usize,
}

impl VersionId {
    pub fn new(namespace: &str, name: &str, version: &str) -> Self {
        let repr = format!("{}-{}-{}", namespace, name, version);
        let name_start = namespace.len() + 1;
        let version_start = name_start + name.len() + 1;
        Self {
            repr,
            name_start,
            version_start,
        }
    }

    pub fn namespace(&self) -> &str {
        &self.repr[..self.name_start - 1]
    }

    pub fn name(&self) -> &str {
        &self.repr[self.name_start..self.version_start - 1]
    }

    pub fn version(&self) -> &str {
        &self.repr[self.version_start..]
    }

    /// Returns an object that, when formatted with `{}`, will produce the URL path for this version.
    ///
    /// ## Example
    ///
    /// ```
    /// let id = thunderstore::VersionId::new("BepInEx", "BepInExPack", "5.4.2100");
    /// assert_eq!(id.path().to_string(), "BepInEx/BepInExPack/5.4.2100");
    /// ```
    pub fn path(&self) -> impl Display + '_ {
        VersionIdPath::new(self)
    }

    /// Consumes the [`VersionId`] and returns the underlying string, formatted as `namespace-name-version`.
    pub fn into_string(self) -> String {
        self.repr
    }

    /// Returns a reference to the underlying string, formatted as `namespace-name-version`.
    pub fn as_str(&self) -> &str {
        &self.repr
    }
}

impl PartialEq for VersionId {
    fn eq(&self, other: &Self) -> bool {
        self.repr == other.repr
    }
}

impl Ord for VersionId {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.repr.cmp(&other.repr)
    }
}

impl PartialOrd for VersionId {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for VersionId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.repr.hash(state);
    }
}

impl AsRef<str> for VersionId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<VersionId> for String {
    fn from(id: VersionId) -> Self {
        id.repr
    }
}

impl Display for VersionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl Debug for VersionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VersionId({})", self.repr)
    }
}

impl TryFrom<String> for VersionId {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
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

impl FromStr for VersionId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        s.to_string().try_into()
    }
}

impl From<(&str, &str, &str)> for VersionId {
    fn from((namespace, name, version): (&str, &str, &str)) -> Self {
        Self::new(namespace, name, version)
    }
}

impl From<(&str, &str, &semver::Version)> for VersionId {
    fn from((namespace, name, version): (&str, &str, &semver::Version)) -> Self {
        Self::new(namespace, name, &version.to_string())
    }
}

impl From<&PackageVersion> for VersionId {
    fn from(pkg: &PackageVersion) -> Self {
        Self::new(&pkg.namespace, &pkg.name, &pkg.version_number.to_string())
    }
}

impl From<&Package> for VersionId {
    fn from(pkg: &Package) -> Self {
        (&pkg.latest).into()
    }
}

struct VersionIdPath<'a> {
    id: &'a VersionId,
}

impl<'a> VersionIdPath<'a> {
    pub fn new(id: &'a VersionId) -> Self {
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

/// A unique identifier for a package, often formatted as `namespace-name`.
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
/// Most methods on [`Client`] accept any type that implements [`IntoPackageId`],
/// which allows any of the above methods to be used interchangeably.
#[derive(Eq, Clone)]
pub struct PackageId {
    repr: String,
    name_start: usize,
}

impl PackageId {
    pub fn new(namespace: &str, name: &str) -> Self {
        let repr = format!("{}-{}", namespace, name);
        let name_start = namespace.len() + 1;
        Self { repr, name_start }
    }

    pub fn namespace(&self) -> &str {
        &self.repr[..self.name_start - 1]
    }

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
    pub fn path(&self) -> impl Display + '_ {
        PackageIdPath::new(self)
    }

    /// Consumes the [`PackageId`] and returns the underlying string, formatted as `namespace-name`.
    pub fn into_string(self) -> String {
        self.repr
    }

    /// Returns a reference to the underlying string, formatted as `namespace-name`.
    pub fn as_str(&self) -> &str {
        &self.repr
    }
}

impl PartialEq for PackageId {
    fn eq(&self, other: &Self) -> bool {
        self.repr == other.repr
    }
}

impl Ord for PackageId {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.repr.cmp(&other.repr)
    }
}

impl PartialOrd for PackageId {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for PackageId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.repr.hash(state);
    }
}

impl AsRef<str> for PackageId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<PackageId> for String {
    fn from(id: PackageId) -> Self {
        id.repr
    }
}

impl Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl Debug for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PackageId({})", self.repr)
    }
}

impl TryFrom<String> for PackageId {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        let mut indices = value.match_indices('-').map(|(i, _)| i);

        let name_start = indices.next().ok_or(Error::InvalidPackageId)? + 1;

        Ok(Self {
            repr: value,
            name_start,
        })
    }
}

impl FromStr for PackageId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        s.to_string().try_into()
    }
}

impl From<(&str, &str)> for PackageId {
    fn from((namespace, name): (&str, &str)) -> Self {
        Self::new(namespace, name)
    }
}

impl From<&Package> for PackageId {
    fn from(pkg: &Package) -> Self {
        Self::new(&pkg.namespace, &pkg.name)
    }
}

struct PackageIdPath<'a> {
    id: &'a PackageId,
}

impl<'a> PackageIdPath<'a> {
    pub fn new(id: &'a PackageId) -> Self {
        Self { id }
    }
}

impl<'a> Display for PackageIdPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.id.namespace(), self.id.name(),)
    }
}

impl From<&VersionId> for PackageId {
    fn from(id: &VersionId) -> Self {
        Self {
            repr: id.repr[..id.version_start].to_string(),
            name_start: id.name_start,
        }
    }
}

pub trait IntoVersionId {
    fn into_id(self) -> Result<VersionId>;
}

impl<T> IntoVersionId for T
where
    T: Into<VersionId>,
{
    fn into_id(self) -> Result<VersionId> {
        Ok(self.into())
    }
}

impl IntoVersionId for String {
    fn into_id(self) -> Result<VersionId> {
        self.try_into()
    }
}

impl IntoVersionId for &str {
    fn into_id(self) -> Result<VersionId> {
        self.parse()
    }
}

pub trait IntoPackageId {
    fn into_id(self) -> Result<PackageId>;
}

impl<T> IntoPackageId for T
where
    T: Into<PackageId>,
{
    fn into_id(self) -> Result<PackageId> {
        Ok(self.into())
    }
}

impl IntoPackageId for String {
    fn into_id(self) -> Result<PackageId> {
        self.try_into()
    }
}

impl IntoPackageId for &str {
    fn into_id(self) -> Result<PackageId> {
        self.parse()
    }
}
