use std::{
    borrow::Cow,
    cmp,
    fmt::{self, Debug, Display},
    hash::Hash,
    str::FromStr,
};

use serde::{Deserialize, Serialize};

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
#[derive(Eq, Clone, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct VersionId {
    repr: Cow<'static, str>,
    name_start: usize,
    version_start: usize,
}

impl VersionId {
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

    pub fn package_id(&self) -> PackageId {
        let repr = Cow::Owned(self.repr[..self.version_start - 1].to_string());

        PackageId {
            repr,
            name_start: self.name_start,
        }
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

impl From<VersionId> for Cow<'static, str> {
    fn from(id: VersionId) -> Self {
        id.into_cow()
    }
}

impl From<VersionId> for String {
    fn from(id: VersionId) -> Self {
        id.into_string()
    }
}

impl Display for VersionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl Debug for VersionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VersionId").field(&self.repr).finish()
    }
}

impl TryFrom<Cow<'static, str>> for VersionId {
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

impl TryFrom<String> for VersionId {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        VersionId::try_from(Cow::Owned(value))
    }
}

impl TryFrom<&'static str> for VersionId {
    type Error = Error;

    fn try_from(value: &'static str) -> Result<Self> {
        VersionId::try_from(Cow::Borrowed(value))
    }
}

impl FromStr for VersionId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        s.to_string().try_into()
    }
}

impl<T, U, V> From<(T, U, V)> for VersionId
where
    T: AsRef<str>,
    U: AsRef<str>,
    V: AsRef<str>,
{
    fn from((namespace, name, version): (T, U, V)) -> Self {
        Self::new(namespace, name, version)
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
#[derive(Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PackageId {
    repr: Cow<'static, str>,
    name_start: usize,
}

impl PackageId {
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

    pub fn with_version(&self, version: impl Display) -> VersionId {
        let repr = Cow::Owned(format!("{}-{}", self.repr, version));
        let version_start = self.repr.len() + 1;

        VersionId {
            repr,
            name_start: self.name_start,
            version_start,
        }
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

impl From<PackageId> for Cow<'static, str> {
    fn from(id: PackageId) -> Self {
        id.into_cow()
    }
}

impl From<PackageId> for String {
    fn from(id: PackageId) -> Self {
        id.into_string()
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

impl TryFrom<Cow<'static, str>> for PackageId {
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

impl TryFrom<String> for PackageId {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        PackageId::try_from(Cow::Owned(value))
    }
}

impl TryFrom<&'static str> for PackageId {
    type Error = Error;

    fn try_from(value: &'static str) -> Result<Self> {
        PackageId::try_from(Cow::Borrowed(value))
    }
}

impl FromStr for PackageId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        s.to_string().try_into()
    }
}

impl<T, U> From<(T, U)> for PackageId
where
    T: AsRef<str>,
    U: AsRef<str>,
{
    fn from((namespace, name): (T, U)) -> Self {
        Self::new(namespace, name)
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
        id.package_id()
    }
}

pub trait IntoVersionId<'a> {
    fn into_id(self) -> Result<Cow<'a, VersionId>>;
}

impl<T> IntoVersionId<'_> for T
where
    T: Into<VersionId>,
{
    fn into_id(self) -> Result<Cow<'static, VersionId>> {
        Ok(Cow::Owned(self.into()))
    }
}

impl<'a> IntoVersionId<'a> for String {
    fn into_id(self) -> Result<Cow<'static, VersionId>> {
        self.try_into().map(Cow::Owned)
    }
}

impl IntoVersionId<'_> for &str {
    fn into_id(self) -> Result<Cow<'static, VersionId>> {
        self.parse().map(Cow::Owned)
    }
}

impl<'a> IntoVersionId<'a> for &'a VersionId {
    fn into_id(self) -> Result<Cow<'a, VersionId>> {
        Ok(Cow::Borrowed(self))
    }
}

pub trait IntoPackageId<'a> {
    fn into_id(self) -> Result<Cow<'a, PackageId>>;
}

impl<T> IntoPackageId<'_> for T
where
    T: Into<PackageId>,
{
    fn into_id(self) -> Result<Cow<'static, PackageId>> {
        Ok(Cow::Owned(self.into()))
    }
}

impl IntoPackageId<'_> for String {
    fn into_id(self) -> Result<Cow<'static, PackageId>> {
        self.try_into().map(Cow::Owned)
    }
}

impl IntoPackageId<'_> for &str {
    fn into_id(self) -> Result<Cow<'static, PackageId>> {
        self.parse().map(Cow::Owned)
    }
}

impl<'a> IntoPackageId<'a> for &'a PackageId {
    fn into_id(self) -> Result<Cow<'a, PackageId>> {
        Ok(Cow::Borrowed(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_id_new() {
        let id = VersionId::new("Kesomannen", "GaleModManager", "0.6.0");
        assert_eq!(id.namespace(), "Kesomannen");
        assert_eq!(id.name(), "GaleModManager");
        assert_eq!(id.version(), "0.6.0");
    }

    #[test]
    fn version_id_path() {
        let id = VersionId::new("notnotnotswipez", "MoreCompany", "1.9.1");
        assert_eq!(id.path().to_string(), "notnotnotswipez/MoreCompany/1.9.1");
    }

    #[test]
    fn parse_version_id() {
        let id: VersionId = "Evaisa-LethalLib-0.16.0".parse().unwrap();
        assert_eq!(id.namespace(), "Evaisa");
        assert_eq!(id.name(), "LethalLib");
        assert_eq!(id.version(), "0.16.0");
    }

    #[test]
    fn version_id_from_tuple() {
        let id: VersionId = ("A", "B", "1.2.3").into();
        assert_eq!(id.namespace(), "A");
        assert_eq!(id.name(), "B");
        assert_eq!(id.version(), "1.2.3");
    }

    #[test]
    fn package_id_new_and_parts() {
        let id = PackageId::new("X", "Y");
        assert_eq!(id.namespace(), "X");
        assert_eq!(id.name(), "Y");
        assert_eq!(id.to_string(), "X-Y");
    }

    #[test]
    fn package_id_from_str() {
        let id: PackageId = "Author-Mod".parse().unwrap();
        assert_eq!(id.namespace(), "Author");
        assert_eq!(id.name(), "Mod");
    }

    #[test]
    fn package_id_path() {
        let id = PackageId::new("NS", "Mod");
        assert_eq!(id.path().to_string(), "NS/Mod");
    }

    #[test]
    fn version_id_into_package_id() {
        let vid = VersionId::new("N", "M", "0.0.1");
        let pid: PackageId = vid.package_id();
        assert_eq!(pid.namespace(), "N");
        assert_eq!(pid.name(), "M");
    }

    #[test]
    fn version_id_serde_roundtrip() {
        let original = VersionId::new("TestAuthor", "TestMod", "1.2.3");
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: VersionId = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn package_id_serde_roundtrip() {
        let original = PackageId::new("TestAuthor", "TestMod");
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: PackageId = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn version_id_serde_from_string() {
        let json = r#""SomeAuthor-SomeMod-0.0.5""#;
        let id: VersionId = serde_json::from_str(json).unwrap();
        assert_eq!(id.namespace(), "SomeAuthor");
        assert_eq!(id.name(), "SomeMod");
        assert_eq!(id.version(), "0.0.5");
    }

    #[test]
    fn package_id_serde_from_string() {
        let json = r#""CoolGuy-ModPack""#;
        let id: PackageId = serde_json::from_str(json).unwrap();
        assert_eq!(id.namespace(), "CoolGuy");
        assert_eq!(id.name(), "ModPack");
    }
}
