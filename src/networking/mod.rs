//! # ficscrape-net
//!
//! A library for networking operations with Archive of Our Own (AO3).
//!
//! This library provides utilities for:
//! - HTTP client configuration with cookie support
//! - Fetching and parsing HTML pages from AO3
//! - AO3 authentication (login, token management)
//! - HTML utilities (selectors, CSS injection)
//!
//! ## Usage - Blocking (default)
//!
//! ```no_run
//! use ao3_api_rs::networking::{create_client, login, get_page};
//!
//! // Create an HTTP client
//! let client = create_client("test").expect("Failed to create client");
//!
//! // Login to AO3
//! login(&client, "log.txt");
//!
//! // Fetch a page
//! let html = get_page("https://archiveofourown.org/works/123456", &client)
//!     .expect("Failed to fetch page");
//! ```

// Module declarations
pub mod auth;
pub mod client;

// Re-export commonly used items for convenience
pub use auth::blocking::{get_token, login};
pub use auth::{LoginInfo, Token, get_login_info};
pub use client::blocking::{create_client, get_init_page, get_page};

// Re-export types from dependencies for convenience
pub use reqwest::Error as NetworkError;
pub use reqwest::blocking::Client;
pub use scraper::error::SelectorErrorKind;
pub use scraper::{Html, Selector};
