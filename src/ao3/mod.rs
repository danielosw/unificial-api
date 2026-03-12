//! # AO3 (Archive of Our Own) Implementation
//!
//! This module provides a complete implementation of the [`Downloader`],
//! [`Extractor`], and [`Authenticator`] traits for
//! [Archive of Our Own](https://archiveofourown.org).
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use unificial_api::ao3::Ao3Client;
//! use unificial_api::traits::{Downloader, Extractor, Authenticator};
//!
//! // Create a client
//! let client = Ao3Client::new("my-user-agent").unwrap();
//!
//! // Optionally log in
//! client.login("credentials.txt").unwrap();
//!
//! // Fetch and extract metadata
//! let html = client.get_all_pages("https://archiveofourown.org/works?page=1").unwrap();
//! ```
//!
//! ## Direct Function Access
//!
//! The underlying functions are also available for direct use:
//!
//! ```rust,no_run
//! use unificial_api::ao3::extraction::extract_fic_metadata;
//! use unificial_api::ao3::networking::{create_client, get_page};
//! ```

use crate::errors::UnificialError;
use crate::traits::{Authenticator, Downloader, Extractor};
use ficdata::{FicMetadata, TagMap};

pub mod extraction;
pub mod networking;

/// AO3 client implementing [`Downloader`], [`Extractor`], and [`Authenticator`].
///
/// Wraps an HTTP client configured for Archive of Our Own, handling
/// rate limiting, redirects, cookie-based sessions, and pagination.
///
/// # Example
///
/// ```rust,no_run
/// use unificial_api::ao3::Ao3Client;
/// use unificial_api::traits::{Downloader, Extractor};
///
/// let client = Ao3Client::new("my-app/1.0").unwrap();
/// let html = client.get_page("https://archiveofourown.org/works/12345").unwrap();
/// let metadata = client.extract_metadata(&html).unwrap();
/// println!("Title: {}", metadata.name);
/// ```
pub struct Ao3Client {
    client: reqwest::blocking::Client,
}

impl Ao3Client {
    /// Create a new AO3 client with the given user agent string.
    pub fn new(useragent: &str) -> Result<Self, UnificialError> {
        Ok(Self {
            client: networking::create_client(useragent)?,
        })
    }

    /// Get a reference to the underlying `reqwest::blocking::Client`.
    pub fn inner(&self) -> &reqwest::blocking::Client {
        &self.client
    }
}

impl Downloader for Ao3Client {
    type Error = UnificialError;

    fn get_page(&self, url: &str) -> Result<String, Self::Error> {
        let response = networking::get_page(url, &self.client)?;
        response.text().map_err(UnificialError::NetworkError)
    }

    fn get_all_pages(&self, url: &str) -> Result<String, Self::Error> {
        let html = networking::get_init_page(url.to_string(), &self.client);
        Ok(html.html())
    }
}

impl Extractor for Ao3Client {
    type Error = UnificialError;

    fn extract_metadata(&self, html: &str) -> Result<FicMetadata, Self::Error> {
        extraction::extract_fic_metadata(html)
    }

    fn extract_tags(&self, html: &str) -> Result<TagMap, Self::Error> {
        extraction::gettags(html.to_string())
    }
}

impl Authenticator for Ao3Client {
    type Error = UnificialError;

    fn login(&self, credentials_path: &str) -> Result<(), Self::Error> {
        networking::login(&self.client, credentials_path);
        Ok(())
    }
}
