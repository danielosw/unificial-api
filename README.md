# ficscrape

HTML extraction and scraping for AO3 fanfiction metadata.

This crate provides functionality to extract metadata from Archive of Our Own
(AO3) HTML pages. It depends on `ficdata` for the core data structures.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ficscrape = "0.1"
```

## Usage

### Extracting Metadata from HTML

```rust
use ficscrape::extract_fic_metadata;

let html = r#"
    <li role="article">
        <h4 class="heading">
            <a href="/works/12345">My Fic Title</a>
        </h4>
        <p class="datetime">15 Jan 2024</p>
        <li class="freeforms"><a class="tag">Fluff</a></li>
    </li>
"#;

if let Ok(metadata) = extract_fic_metadata(html) {
    println!("Fic ID: {}", metadata.id);
    println!("Title: {}", metadata.name);
    println!("URL: {}", metadata.url);
    println!("Last Updated: {}", metadata.last_updated);
}
```

### Extracting Tags

```rust
use ficscrape::gettags;

let html = r#"
    <li class="freeforms"><a class="tag">Fluff</a></li>
    <li class="freeforms"><a class="tag">Angst</a></li>
    <li class="warnings"><a class="tag">No Warnings</a></li>
"#;

if let Ok(tags) = gettags(html.to_string()) {
    println!("Freeform tags: {:?}", tags.get("freeforms"));
    println!("Warning tags: {:?}", tags.get("warnings"));
}
```

## API

- `extract_fic_metadata(html: &str) -> Result<FicMetadata, Ao3ApiError>` -
  Extract metadata from AO3 HTML
- `gettags(html: String) -> Result<TagMap, Ao3ApiError>` - Extract tags from
  HTML

## Related Crates

- [`ficdata`](https://crates.io/crates/ficdata) - Core data structures and
  persistence for AO3 fanfiction metadata
