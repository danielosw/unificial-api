# unificial-api

A trait-based fanfiction API framework with pluggable site downloaders.

This crate defines core traits (`Downloader`, `Extractor`, `Authenticator`)
that abstract over common fanfiction site operations such as page fetching,
metadata extraction, and authentication. Implement the traits to add support
for any site — AO3 is included as the built-in example.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
unificial-api = { git = "https://github.com/danielosw/unificial-api.git" }
```

## Quick Start (AO3)

```rust
use unificial_api::ao3::Ao3Client;
use unificial_api::traits::{Downloader, Extractor, Authenticator};

// Create a client
let client = Ao3Client::new("my-app/1.0").unwrap();

// Optionally authenticate
client.login("credentials.txt").unwrap();

// Fetch a page and extract metadata
let html = client.get_page("https://archiveofourown.org/works/12345").unwrap();
let metadata = client.extract_metadata(&html).unwrap();
println!("Title: {}", metadata.name);
```

## Core Traits

### `Downloader`

Fetch pages from a fanfiction site with built-in pagination support.

```rust
pub trait Downloader {
    type Error: std::error::Error;
    fn get_page(&self, url: &str) -> Result<String, Self::Error>;
    fn get_all_pages(&self, url: &str) -> Result<String, Self::Error>;
}
```

### `Extractor`

Parse HTML into structured `FicMetadata` and `TagMap` data (from the
[`ficdata`](https://github.com/danielosw/ficdata) crate).

```rust
pub trait Extractor {
    type Error: std::error::Error;
    fn extract_metadata(&self, html: &str) -> Result<FicMetadata, Self::Error>;
    fn extract_tags(&self, html: &str) -> Result<TagMap, Self::Error>;
}
```

### `Authenticator`

Log in to a site for access to restricted content.

```rust
pub trait Authenticator {
    type Error: std::error::Error;
    fn login(&self, credentials_path: &str) -> Result<(), Self::Error>;
}
```

## Implementing a New Downloader

To support a new site, create a struct and implement the traits you need:

```rust
use unificial_api::traits::{Downloader, Extractor};
use unificial_api::errors::UnificialError;
use ficdata::{FicMetadata, TagMap};

struct MySiteClient { /* fields */ }

impl Downloader for MySiteClient {
    type Error = UnificialError;
    fn get_page(&self, url: &str) -> Result<String, Self::Error> { todo!() }
    fn get_all_pages(&self, url: &str) -> Result<String, Self::Error> { todo!() }
}

impl Extractor for MySiteClient {
    type Error = UnificialError;
    fn extract_metadata(&self, html: &str) -> Result<FicMetadata, Self::Error> { todo!() }
    fn extract_tags(&self, html: &str) -> Result<TagMap, Self::Error> { todo!() }
}
```

## AO3 Module

The `ao3` module provides a full `Ao3Client` that implements all three traits.
Lower-level functions are also available for direct use:

- `unificial_api::ao3::extraction::extract_fic_metadata` — extract metadata
  from an AO3 HTML fragment
- `unificial_api::ao3::extraction::gettags` — extract tags from HTML
- `unificial_api::ao3::networking::create_client` — create a configured HTTP
  client
- `unificial_api::ao3::networking::get_page` — fetch a single page with retry
  logic
- `unificial_api::ao3::networking::get_init_page` — fetch all paginated pages

## Related Crates

- [`ficdata`](https://github.com/danielosw/ficdata) — Core data structures
  (`FicMetadata`, `TagMap`) and persistence utilities
