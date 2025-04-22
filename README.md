# Thunderstore Client

A library for interacting with the Thunderstore API.

See the [crate docs](https://docs.rs/thunderstore/latest/thunderstore) for more information.

## Examples

```rs
#[tokio::main]
async fn main() -> thunderstore::Result<()> {
    let client = thunderstore::Client::builder()
        .with_token("tss_XXX")
        .build()?;

    let package = client.get_package(("Kesomannen", "GaleModManager")).await?;
    let _bytes = client.download(package.latest.ident).await?;

    client.publish_file(
        "CoolMod.zip",
        PackageMetadata::new("Kesomannen", ["lethal-company"])
            .with_global_categories(["tools"])
            .with_categories("lethal-company", ["serverside"])
    ).await?;
}
```
