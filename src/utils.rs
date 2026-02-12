//! Utility functions for HTML processing and shared helpers

use crate::errors::Ao3ApiError;
use regex::Regex;
use scraper::Selector;
use std::sync::LazyLock;
use std::sync::{Arc, Mutex};

/// Creates a selector from provided string
///
/// Internal utility function for parsing CSS selectors.
#[inline(always)]
pub(crate) fn make_selector(
    selector: &str,
) -> Result<Selector, scraper::error::SelectorErrorKind<'_>> {
    Selector::parse(selector)
}

/// Wraps argument in Arc<Mutex<T>> for thread-safe shared ownership
///
/// Internal utility function for creating thread-safe shared state.
#[inline(always)]
pub(crate) fn arcify<T>(arg: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(arg))
}
/// Macro to create a static LazyLock

#[macro_export]
macro_rules! make_static {
    ($expr:expr) => {{ LazyLock::new(|| $expr) }};
}
/// Macro to select raw text from HTML document

#[macro_export]
macro_rules! select_raw_text {
    ($document:expr, $selector:expr) => {
        // Attempt to select elements
        $document
            .select($selector)
            .map(|elem| elem.text().collect::<String>().trim().to_string())
    };
}
/// Macro to select raw text from HTML document, returning only the next element
#[macro_export]
macro_rules! select_raw_text_next {
    ($document:expr, $selector:expr) => {
        // Attempt to select elements
        $document
            .select($selector)
            .next()
            .map(|elem| elem.text().collect::<String>().trim().to_string())
    };
}
/// Macro to select text from HTML document, returning an Vec in an Option
#[macro_export]
macro_rules! select_text {
    ($document:expr, $selector:expr) => {{
        // Attempt to select elements
        let result = select_raw_text!($document, $selector).collect::<Vec<String>>();

        // Return an Option, None if result is empty
        result
    }};
}
pub(crate) fn safe_static_selector(
    selector: Option<Selector>,
    backup: &str,
) -> Result<Selector, Ao3ApiError> {
    selector.map(Ok).unwrap_or_else(|| {
        make_selector(backup)
            .map_err(|_| Ao3ApiError::SelectorError("Failed to create CSS selector".to_string()))
    })
}

pub(crate) fn safe_static_regex(
    regex: Option<regex::Regex>,
    backup: &str,
) -> Result<Regex, Ao3ApiError> {
    regex.map(Ok).unwrap_or_else(|| {
        Regex::new(backup)
            .map_err(|_| Ao3ApiError::RegexError("Failed to compile regex".to_string()))
    })
}

#[macro_export]
macro_rules! define_selector {
    ($name:ident, $name_text:ident, $text:expr) => {
        static $name_text: &str = $text;

        static $name: LazyLock<Option<Selector>> = make_static!(make_selector($text).ok());
    };
}
#[macro_export]
macro_rules! define_regex {
    ($name:ident, $name_text:ident, $text:expr) => {
        static $name_text: &str = $text;

        static $name: LazyLock<std::option::Option<regex::Regex>> =
            make_static!({ Regex::new($text).ok() });
    };
}
