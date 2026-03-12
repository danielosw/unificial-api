//! Core traits for the unificial-api framework.
//!
//! These traits define the interface that site-specific implementations must
//! provide. By implementing these traits, you can add support for any
//! fanfiction site.
//!
//! # Traits
//!
//! - [`Downloader`] — Fetching pages from a site (with pagination support)
//! - [`Extractor`] — Extracting fanfiction metadata from HTML content
//! - [`Authenticator`] — Logging in to a site
//!
//! # Example
//!
//! See the [`ao3`](crate::ao3) module for a complete implementation of all
//! three traits targeting Archive of Our Own.

use ficdata::TagMap;

/// Type alias for fanfiction metadata.
///
/// Re-exported from [`ficdata::FicMetadata`] under a shorter name for
/// convenience across site implementations.
pub type Metadata = ficdata::FicMetadata;

/// A site downloader capable of fetching pages.
///
/// Implementors handle HTTP requests, rate limiting, redirect following,
/// and pagination for a specific fanfiction site.
///
/// # Example
///
/// ```rust,no_run
/// use unificial_api::ao3::Ao3Client;
/// use unificial_api::traits::Downloader;
///
/// let client = Ao3Client::new("my-user-agent").unwrap();
/// let html = client.get_page("https://archiveofourown.org/works/12345").unwrap();
/// ```
pub trait Downloader {
    type Error: std::error::Error;

    /// Fetch a single page and return its body as a string.
    fn get_page(&self, url: &str) -> Result<String, Self::Error>;

    /// Fetch a page and handle pagination, returning all pages combined.
    fn get_all_pages(&self, url: &str) -> Result<String, Self::Error>;
}

/// An extractor that parses fanfiction metadata from HTML content.
///
/// Implementors know how to parse a specific site's HTML structure and
/// extract structured [`Metadata`] and [`TagMap`] data from it.
///
/// # Example
///
/// ```rust,no_run
/// use unificial_api::ao3::Ao3Client;
/// use unificial_api::traits::Extractor;
///
/// let client = Ao3Client::new("my-user-agent").unwrap();
/// let html = "<li role=\"article\">...</li>";
/// let metadata = client.extract_metadata(html).unwrap();
/// ```
pub trait Extractor {
    type Error: std::error::Error;

    /// Extract metadata for a single fic from an HTML fragment.
    fn extract_metadata(&self, html: &str) -> Result<Metadata, Self::Error>;

    /// Extract tags from an HTML fragment.
    fn extract_tags(&self, html: &str) -> Result<TagMap, Self::Error>;
}

/// An authenticator that can log in to a fanfiction site.
///
/// Implementors handle token retrieval and credential submission
/// for a specific site's login flow.
///
/// # Example
///
/// ```rust,no_run
/// use unificial_api::ao3::Ao3Client;
/// use unificial_api::traits::Authenticator;
///
/// let client = Ao3Client::new("my-user-agent").unwrap();
/// client.login("credentials.txt").unwrap();
/// ```
pub trait Authenticator {
    type Error: std::error::Error;

    /// Log in to the site using credentials from the given file path.
    fn login(&self, credentials_path: &str) -> Result<(), Self::Error>;
}
