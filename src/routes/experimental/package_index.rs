use async_stream::try_stream;
use futures_core::Stream;
use futures_util::TryStreamExt;

use crate::{models::PackageIndexEntry, prelude::*, Result};

impl Client {
    pub async fn stream_package_index(
        &self,
    ) -> Result<impl Stream<Item = Result<PackageIndexEntry>>> {
        let url = self.url("/experimental/package-index");

        let mut buffer = String::new();

        let mut stream = self.get(url).await?.bytes_stream();

        Ok(try_stream! {
            while let Some(chunk) = stream.try_next().await? {
                let str = str::from_utf8(&chunk).expect("invalid UTF-8 received by thunderstore");
                let newlines = str.match_indices('\n');

                if newlines.clone().next().is_none() {
                    buffer.push_str(str);
                } else {
                    let mut last_char_index = 0;

                    for (i, (char_index, _)) in newlines.enumerate() {
                        let slice = match i {
                            0 if buffer.len() > 0 => {
                                buffer.push_str(&str[..char_index]);
                                buffer.as_str()
                            }
                            _ => &str[last_char_index..char_index],
                        };

                        yield serde_json::from_str(slice)?;
                        last_char_index = char_index;
                    }

                    buffer.clear();
                    buffer.push_str(&str[last_char_index + 1..]);
                }
            }
        })
    }
}
