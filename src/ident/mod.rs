pub mod package;
pub mod version;

pub use package::*;
pub use version::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_id_new() {
        let id = VersionIdent::new("Kesomannen", "GaleModManager", "0.6.0");
        assert_eq!(id.namespace(), "Kesomannen");
        assert_eq!(id.name(), "GaleModManager");
        assert_eq!(id.version(), "0.6.0");
    }

    #[test]
    fn version_id_path() {
        let id = VersionIdent::new("notnotnotswipez", "MoreCompany", "1.9.1");
        assert_eq!(id.path().to_string(), "notnotnotswipez/MoreCompany/1.9.1");
    }

    #[test]
    fn parse_version_id() {
        let id: VersionIdent = "Evaisa-LethalLib-0.16.0".parse().unwrap();
        assert_eq!(id.namespace(), "Evaisa");
        assert_eq!(id.name(), "LethalLib");
        assert_eq!(id.version(), "0.16.0");
    }

    #[test]
    fn version_id_from_tuple() {
        let id: VersionIdent = ("A", "B", "1.2.3").into();
        assert_eq!(id.namespace(), "A");
        assert_eq!(id.name(), "B");
        assert_eq!(id.version(), "1.2.3");
    }

    #[test]
    fn package_id_new_and_parts() {
        let id = PackageIdent::new("X", "Y");
        assert_eq!(id.namespace(), "X");
        assert_eq!(id.name(), "Y");
        assert_eq!(id.to_string(), "X-Y");
    }

    #[test]
    fn package_id_from_str() {
        let id: PackageIdent = "Author-Mod".parse().unwrap();
        assert_eq!(id.namespace(), "Author");
        assert_eq!(id.name(), "Mod");
    }

    #[test]
    fn package_id_path() {
        let id = PackageIdent::new("NS", "Mod");
        assert_eq!(id.path().to_string(), "NS/Mod");
    }

    #[test]
    fn version_id_into_package_id() {
        let vid = VersionIdent::new("N", "M", "0.0.1");
        let pid: PackageIdent = vid.package_id();
        assert_eq!(pid.namespace(), "N");
        assert_eq!(pid.name(), "M");
    }

    #[test]
    fn version_id_serde_roundtrip() {
        let original = VersionIdent::new("TestAuthor", "TestMod", "1.2.3");
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: VersionIdent = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn package_id_serde_roundtrip() {
        let original = PackageIdent::new("TestAuthor", "TestMod");
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: PackageIdent = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn version_id_serde_from_string() {
        let json = r#""SomeAuthor-SomeMod-0.0.5""#;
        let id: VersionIdent = serde_json::from_str(json).unwrap();
        assert_eq!(id.namespace(), "SomeAuthor");
        assert_eq!(id.name(), "SomeMod");
        assert_eq!(id.version(), "0.0.5");
    }

    #[test]
    fn package_id_serde_from_string() {
        let json = r#""CoolGuy-ModPack""#;
        let id: PackageIdent = serde_json::from_str(json).unwrap();
        assert_eq!(id.namespace(), "CoolGuy");
        assert_eq!(id.name(), "ModPack");
    }
}
