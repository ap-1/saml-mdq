# saml-mdq

A Rust client for the [SAML Metadata Query (MDQ) Protocol](https://datatracker.ietf.org/doc/draft-young-md-query-saml/), used to fetch SAML metadata for individual entities from an MDQ server.

## Usage

```rust
use saml_mdq::MdqClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = MdqClient::builder("https://mdq.incommon.org").build()?;

    let metadata = client
        .fetch_entity("https://login.cmu.edu/idp/shibboleth")
        .await?;

    println!("Entity ID: {:?}", metadata.entity_id);
    Ok(())
}
```

## Caching

An optional in-memory cache can be configured to avoid redundant network requests:

```rust
use saml_mdq::{MdqCache, MdqClient};
use std::time::Duration;

let cache = MdqCache::new(1000, Duration::from_secs(3600));
let client = MdqClient::builder("https://mdq.incommon.org")
    .cache(cache)
    .build()?;
```

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE-2.0)
- [MIT License](LICENSE-MIT)

at your option.
