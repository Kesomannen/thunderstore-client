# Rust Thunderstore Client

A library for interacting with the Thunderstore API.

## Example

```rs
#[tokio::main]
async fn main() -> thunderstore::Result<()> {
    let client = thunderstore::Client::builder()
        .with_token("tss_XXX")
        .build()?;

    let package = client.get_package(("Kesomannen", "GaleModManager")).await?;
     client.download_to_dir(&package.latest, r"C:\Users\bobbo\Downloads").await?;

     client.publish_file(
         "CoolMod.zip",
         PackageMetadata::new("Kesomannen", ["lethal-company"])
             .with_global_categories(["tools"])
             .with_categories("lethal-company", ["serverside"])
     ).await?;
 }
```
