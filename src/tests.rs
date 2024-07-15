use super::*;

#[test]
fn version_id_new_works() {
    let id = VersionId::new("Kesomannen", "GaleModManager", "0.6.0");
    assert_eq!(id.namespace(), "Kesomannen");
    assert_eq!(id.name(), "GaleModManager");
    assert_eq!(id.version(), "0.6.0"); 
}

#[test]
fn version_id_path_works() {
    let id = VersionId::new("notnotnotswipez", "MoreCompany", "1.9.1");
    assert_eq!(id.path().to_string(), "notnotnotswipez/MoreCompany/1.9.1");
}

#[test]
fn parse_version_id_works() {
    let id: VersionId = "Evaisa-LethalLib-0.16.0".parse().unwrap();
    assert_eq!(id.namespace(), "Evaisa");
    assert_eq!(id.name(), "LethalLib");
    assert_eq!(id.version(), "0.16.0");
}

#[tokio::test]
async fn get_package_index_works() -> Result<()> {
    Client::new().get_package_index().await?;
    Ok(())
}

#[tokio::test]
async fn get_package_works() -> Result<()> {
    let client = Client::new();

    client.get_package(("Kesomannen", "GaleModManager")).await?;
    client.get_package("Kesomannen-GaleModManager").await?;

    Ok(())
}

#[tokio::test]
async fn get_package_fails_when_not_found() -> Result<()> {
    let client = Client::new();

    match client.get_package(("Kesomannen", "GaleModManager2")).await {
        Err(Error::NotFound) => (),
        other => panic!("expected NotFound error, got {:?}", other),
    }

    Ok(())
}

#[tokio::test]
async fn get_version_works() -> Result<()> {
    let client = Client::new();

    client
        .get_version(("Kesomannen", "GaleModManager", "0.6.0"))
        .await?;
    client
        .get_version((
            "Kesomannen",
            "GaleModManager",
            &semver::Version::new(0, 6, 0),
        ))
        .await?;
    client
        .get_version("Kesomannen-GaleModManager-0.6.0")
        .await?;

    Ok(())
}

#[tokio::test]
async fn get_changelog_works() -> Result<()> {
    Client::new()
        .get_changelog(("Kesomannen", "GaleModManager", "0.1.0"))
        .await?;
    Ok(())
}

#[tokio::test]
async fn get_readme_works() -> Result<()> {
    Client::new()
        .get_readme(("Kesomannen", "GaleModManager", "0.1.0"))
        .await?;
    Ok(())
}
