//! # unificial-api
//!
//! A trait-based fanfiction API framework with pluggable site downloaders.
//!
//! This crate defines core traits ([`Downloader`](traits::Downloader),
//! [`Extractor`](traits::Extractor), [`Authenticator`](traits::Authenticator))
//! that abstract over fanfiction site operations. Site-specific implementations
//! live in their own modules — see [`ao3`] for the built-in AO3 support.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use unificial_api::ao3::Ao3Client;
//! use unificial_api::traits::{Downloader, Extractor, Authenticator};
//!
//! let client = Ao3Client::new("my-app/1.0").unwrap();
//!
//! // Authenticate (optional)
//! client.login("credentials.txt").unwrap();
//!
//! // Fetch pages and extract metadata
//! let html = client.get_page("https://archiveofourown.org/works/12345").unwrap();
//! let metadata = client.extract_metadata(&html).unwrap();
//! println!("Title: {}", metadata.name);
//! ```
//!
//! ## Implementing a New Downloader
//!
//! To add support for a new site, implement the traits from [`traits`]:
//!
//! ```rust,no_run
//! use unificial_api::traits::{Downloader, Extractor};
//! use unificial_api::errors::UnificialError;
//! use ficdata::TagMap;
//! use unificial_api::traits::Metadata;
//!
//! struct MySiteClient { /* ... */ }
//!
//! impl Downloader for MySiteClient {
//!     type Error = UnificialError;
//!     fn get_page(&self, url: &str) -> Result<String, Self::Error> { todo!() }
//!     fn get_all_pages(&self, url: &str) -> Result<String, Self::Error> { todo!() }
//! }
//!
//! impl Extractor for MySiteClient {
//!     type Error = UnificialError;
//!     fn extract_metadata(&self, html: &str) -> Result<Metadata, Self::Error> { todo!() }
//!     fn extract_tags(&self, html: &str) -> Result<TagMap, Self::Error> { todo!() }
//! }
//! ```

pub mod errors;
pub mod traits;
pub mod ao3;
mod utils;
