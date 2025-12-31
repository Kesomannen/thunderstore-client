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

/// A unique identifier for a package version, formatted as `namespace-name-version`.
///
/// This struct can be created in a number of ways:
/// ```
/// use thunderstore::VersionIdent;
///
/// let a = VersionIdent::new("BepInEx", "BepInExPack", "5.4.2100");
/// let b: VersionIdent = "BepInEx-BepInExPack-5.4.2100".parse().unwrap();
/// let c: VersionIdent = ("BepInEx", "BepInExPack", "5.4.2100").into();
/// ```
///
/// Methods on [`crate::Client`] accept any type that implements [`IntoVersionIdent`],
/// which allows any of the above methods to be used interchangeably.
///
/// The underlying string is either an owned [`String`] or a string literal (`&'static str`).
#[derive(Eq, Clone, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct VersionIdent {
    pub(super) repr: Cow<'static, str>,
    pub(super) name_start: usize,
    pub(super) version_start: usize,
}

impl VersionIdent {
    /// Creates a new [`VersionIdent`].
    ///
    /// This copies the arguments into a newly allocated `String`, delimited by `-`.
    ///
    /// ## Examples
    ///
    /// ```
    /// use thunderstore::VersionIdent;
    ///
    /// let ident = VersionIdent::new("BepInEx", "BepInExPack", "5.4.2100");
    /// assert_eq!(ident.into_string(), "BepInEx-BepInExPack-5.4.2100");
    /// ```
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

    /// The namespace/owner of the package.
    ///
    /// ## Examples
    ///
    /// ```
    /// use thunderstore::VersionIdent;
    ///
    /// let ident: VersionIdent = "BepInEx-BepInExPack-5.4.2100".parse().unwrap();
    /// assert_eq!(ident.namespace(), "BepInEx");
    /// ```
    #[inline]
    pub fn namespace(&self) -> &str {
        &self.repr[..self.name_start - 1]
    }

    /// The name of the package.
    ///
    /// ## Examples
    ///
    /// ```
    /// use thunderstore::VersionIdent;
    ///
    /// let ident: VersionIdent = "BepInEx-BepInExPack-5.4.2100".parse().unwrap();
    /// assert_eq!(ident.name(), "BepInExPack");
    /// ```
    #[inline]
    pub fn name(&self) -> &str {
        &self.repr[self.name_start..self.version_start - 1]
    }

    /// The version number of the package.
    ///
    /// ## Examples
    ///
    /// ```
    /// use thunderstore::VersionIdent;
    ///
    /// let ident: VersionIdent = "BepInEx-BepInExPack-5.4.2100".parse().unwrap();
    /// assert_eq!(ident.version(), "5.4.2100");
    /// ```
    #[inline]
    pub fn version(&self) -> &str {
        &self.repr[self.version_start..]
    }

    /// The version number of the package as a [`semver::Version`]
    ///
    /// ## Examples
    ///
    /// ```
    /// use thunderstore::VersionIdent;
    ///
    /// let ident: VersionIdent = "BepInEx-BepInExPack-5.4.2100".parse().unwrap();
    ///
    /// let version = ident.parsed_version();
    /// assert_eq!(version.major, 5);
    /// assert_eq!(version.minor, 4);
    /// assert_eq!(version.patch, 2100);
    /// ```
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
    /// use thunderstore::VersionIdent;
    ///
    /// let id = VersionIdent::new("BepInEx", "BepInExPack", "5.4.2100");
    /// assert_eq!(id.path().to_string(), "BepInEx/BepInExPack/5.4.2100");
    /// ```
    #[inline]
    pub fn path(&self) -> impl Display + '_ {
        VersionIdPath::new(self)
    }

    /// Unwraps the underlying string, formatted as `namespace-name-version`.
    #[inline]
    pub fn into_cow(self) -> Cow<'static, str> {
        self.repr
    }

    /// Unwraps the underlying string, formatted as `namespace-name-version`.
    ///
    /// If the string is a `'static str`, it is converted to an owned `String`. If you don't want
    /// that, see [`VersionIdent::into_cow`].
    #[inline]
    pub fn into_string(self) -> String {
        self.repr.into_owned()
    }

    /// Returns a reference to the underlying string, formatted as `namespace-name-version`.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.repr
    }

    /// Returns a copy of the identifier with the version removed.
    pub fn package_id(&self) -> PackageIdent {
        let repr = match &self.repr {
            Cow::Borrowed(str) => Cow::Borrowed(&str[..self.version_start - 1]),
            Cow::Owned(str) => Cow::Owned(str[..self.version_start - 1].to_string()),
        };

        PackageIdent {
            repr,
            name_start: self.name_start,
        }
    }

    pub fn eq_package(&self, other: &PackageIdent) -> bool {
        self.namespace() == other.namespace() && self.name() == other.name()
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

        let name_start = indices.next().ok_or(Error::InvalidIdent)? + 1;
        let version_start = indices.next().ok_or(Error::InvalidIdent)? + 1;

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

impl Display for VersionIdPath<'_> {
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

/// A fallible conversion to [`Cow<'a, VersionIdent>`].
///
/// This is used in methods on [`crate::Client`] to add flexibility in the argument types.
///
/// This usually clones the input, unless you pass it a reference an already constructed [`VersionIdent`],
/// in which case no copying will be performed.
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

impl IntoVersionIdent<'_> for String {
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
