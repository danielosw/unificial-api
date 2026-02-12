//! # ficscrape
//!
//! HTML extraction and scraping for AO3 fanfiction metadata.
//!
//! This crate provides functionality to extract metadata from AO3 HTML pages.
//! It depends on `ficdata` for the core data structures.
//!
//! ## Usage
//!
//! ```rust,no_run
//!	  use ao3_api_rs::extraction::extract_fic_metadata;
//! let html = "<li role=\"article\">...</li>";
//! if let Ok(metadata) = extract_fic_metadata(html) {
//!     println!("Found fic: {}", metadata.name);
//! }
//! ```
use crate::errors::Ao3ApiError;
use crate::utils::{make_selector, safe_static_regex, safe_static_selector};
use crate::{
    define_regex, define_selector, make_static, select_raw_text, select_raw_text_next, select_text,
};
use ficdata::{FicMetadata, TagMap};
use regex::Regex;
use scraper::{Html, selector::Selector};
use std::collections::HashMap;
use std::num::ParseIntError;
use std::sync::LazyLock;
// TODO: convert from macros to const functions for better error handling making working on this file less of a mess
define_regex!(
    TAG_REGEX,
    TAG_REGEX_TEXT,
    r#"<li class="(.*?)">.*?<a class="tag".*?">(.*?)</a"#
);
define_regex!(FIC_ID_REGEX, FIC_ID_REGEX_TEXT, r#"/works/(\d+)"#);
define_selector!(
    HEADING_SELECTOR,
    HEADING_SELECTOR_TEXT,
    r#"h4[class="heading"]"#
);
define_selector!(LINK_SELECTOR, LINK_SELECTOR_TEXT, "a");
define_selector!(
    DATETIME_SELECTOR,
    DATETIME_SELECTOR_TEXT,
    r#"p[class="datetime"]"#
);
define_selector!(AUTHOR_SELECTOR, AUTHOR_SELECTOR_TEXT, r#"a[rel="author"]"#);
define_selector!(SERIES_SELECTOR, SERIES_SELECTOR_TEXT, r#"ul.series li"#);
define_selector!(
    USER_STUFF_SELECTOR,
    USER_STUFF_SELECTOR_TEXT,
    r#"blockquote.userstuff.summary > p"#
);
define_selector!(
    USER_STUFF_SELECTOR_BACKUP,
    USER_STUFF_SELECTOR_BACKUP_TEXT,
    r#"blockquote.userstuff.summary"#
);
define_selector!(
    FANDOM_SELECTOR,
    FANDOM_SELECTOR_TEXT,
    r#"h5.fandoms.heading a.tag"#
);
define_selector!(
    SHIP_TYPE_SELECTOR,
    SHIP_TYPE_SELECTOR_TEXT,
    r#"a.help.symbol[href="/help/symbols-key.html"] span.category span.text"#
);
define_selector!(LANGUAGE_SELECTOR, LANGUAGE_SELECTOR_TEXT, r#"dd.language"#);
define_selector!(CHAPTERS_SELECTOR, CHAPTERS_SELECTOR_TEXT, r#"dd.chapters"#);
define_selector!(KUDOS_SELECTOR, KUDOS_SELECTOR_TEXT, r#"dd.kudos"#);
define_selector!(WORDS_SELECTOR, WORDS_SELECTOR_TEXT, r#"dd.words"#);
define_selector!(HITS_SELECTOR, HITS_SELECTOR_TEXT, r#"dd.hits"#);
/// Gets all fic tags from passed in String
///
/// # Arguments
/// * `fic` - a string containing the html of the fic's display area
///
/// # Returns
/// * returns a HashMap mapping tag categories to vectors of tag values
pub fn gettags(fic: String) -> Result<TagMap, Ao3ApiError> {
    let mut tags: HashMap<String, Vec<String>> = HashMap::new();

    safe_static_regex(TAG_REGEX.clone(), &TAG_REGEX_TEXT)?
        .captures_iter(&fic)
        .for_each(|cap| {
            if let (Some(category), Some(value)) = (cap.get(1), cap.get(2)) {
                tags.entry(category.as_str().to_string())
                    .or_default()
                    .push(value.as_str().to_string());
            }
        });

    Ok(tags)
}

fn parse_number_with_commas(text: &str) -> Result<u32, ParseIntError> {
    text.replace(',', "").parse::<u32>()
}

/// Helper function to extract series list from HTML document
fn extract_series_list(document: &Html) -> Result<Vec<String>, Ao3ApiError> {
    if SERIES_SELECTOR.is_none() {
        return Ok(Vec::new());
    }

    Ok(document
        .select(&safe_static_selector(
            SERIES_SELECTOR.clone(),
            SERIES_SELECTOR_TEXT,
        )?)
        .map(|elem| {
            // Extract the entire text content of the li element
            // This will give us something like "Part 10 of Series Name"
            let text = elem.text().collect::<String>();
            // Clean up extra whitespace
            text.split_whitespace().collect::<Vec<_>>().join(" ")
        })
        .filter(|s: &String| !s.is_empty())
        .collect::<Vec<String>>())
}

/// Extract fic metadata from HTML
pub fn extract_fic_metadata(item: &str) -> Result<FicMetadata, Ao3ApiError> {
    let document = Html::parse_document(item);

    let desc: String = document
        .select(&safe_static_selector(
            USER_STUFF_SELECTOR.clone(),
            USER_STUFF_SELECTOR_TEXT,
        )?)
        .next()
        .unwrap_or(
            document
                .select(&safe_static_selector(
                    USER_STUFF_SELECTOR_BACKUP.clone(),
                    USER_STUFF_SELECTOR_BACKUP_TEXT,
                )?)
                .next()
                .ok_or(Ao3ApiError::SelectorError(
                    "Next failed to run when getting description".to_string(),
                ))?,
        )
        .text()
        .collect();
    // Get fic name and URL from heading
    let heading = document
        .select(&safe_static_selector(
            HEADING_SELECTOR.clone(),
            HEADING_SELECTOR_TEXT,
        )?)
        .next()
        .ok_or(Ao3ApiError::SelectorError(
            "Next failed to run when getting heading".to_string(),
        ))?;
    let link = heading
        .select(&safe_static_selector(
            LINK_SELECTOR.clone(),
            LINK_SELECTOR_TEXT,
        )?)
        .next()
        .ok_or(Ao3ApiError::SelectorError(
            "Next failed to run when getting link".to_string(),
        ))?;
    let url = "https://archiveofourown.org".to_owned()
        + link.attr("href").ok_or(Ao3ApiError::SelectorError(
            "Failed to get href attribute from link".to_string(),
        ))?;
    let name = link.text().collect::<String>().trim().to_string();

    // Extract ID from URL using compiled regex
    let id = safe_static_regex(FIC_ID_REGEX.clone(), FIC_ID_REGEX_TEXT)?
        .captures(&url)
        .ok_or(Ao3ApiError::RegexError(
            "Failed to capture id from url".to_string(),
        ))?
        .get(1)
        .ok_or(Ao3ApiError::RegexError(
            "Failed to get id from url".to_string(),
        ))?
        .as_str()
        .to_string();

    // Get tags
    let tags = gettags(item.to_string()).unwrap_or_default();

    // Get last updated date
    let last_updated = document
        .select(&safe_static_selector(
            DATETIME_SELECTOR.clone(),
            DATETIME_SELECTOR_TEXT,
        )?)
        .next()
        .and_then(|elem| elem.text().next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    // Extract author usernames
    let authors: Vec<String> = document
        .select(&safe_static_selector(
            AUTHOR_SELECTOR.clone(),
            AUTHOR_SELECTOR_TEXT,
        )?)
        .map(|elem| elem.text().collect::<String>().trim().to_string())
        .collect();

    // Extract fandoms from <h5 class="fandoms heading"> structure
    let fandom: Vec<String> = select_text!(
        document,
        &safe_static_selector(FANDOM_SELECTOR.clone(), FANDOM_SELECTOR_TEXT)?
    );

    // Extract ship type from category span
    // Look for the help symbol link that contains the category information
    // Extract the text content and split by comma to get a list of ship types
    let ship_type: Vec<String> = select_text!(
        document,
        &safe_static_selector(SHIP_TYPE_SELECTOR.clone(), SHIP_TYPE_SELECTOR_TEXT)?
    )
    .iter()
    .flat_map(|text| {
        text.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
    })
    .collect();

    // Extract language from dd.language
    let language = select_raw_text_next!(
        document,
        &safe_static_selector(LANGUAGE_SELECTOR.clone(), LANGUAGE_SELECTOR_TEXT)?
    );

    // Extract chapters from dd.chapters
    let chapters = select_raw_text_next!(
        document,
        &safe_static_selector(CHAPTERS_SELECTOR.clone(), CHAPTERS_SELECTOR_TEXT)?
    );

    // Extract kudos from dd.kudos
    // The kudos might be in a link or directly in the dd
    let kudos = parse_number_with_commas(
        select_raw_text_next!(
            document,
            &safe_static_selector(KUDOS_SELECTOR.clone(), KUDOS_SELECTOR_TEXT)?
        )
        .ok_or(Ao3ApiError::SelectorError(
            "Failed to select kudos from dd.kudos".to_string(),
        ))?
        .trim(),
    );
    // Extract words from dd.words
    let words = parse_number_with_commas(
        select_raw_text_next!(
            document,
            &safe_static_selector(WORDS_SELECTOR.clone(), WORDS_SELECTOR_TEXT)?
        )
        .ok_or(Ao3ApiError::SelectorError(
            "Failed to extract words from dd.words".to_string(),
        ))?
        .trim(),
    );

    // Extract series from ul.series
    // Format: "Part <strong>10</strong> of <a href="/series/1301696">Series Name</a>"
    let series = extract_series_list(&document).unwrap_or_default();

    // Extract hits from dd.hits
    let hits = parse_number_with_commas(
        select_raw_text_next!(
            document,
            &safe_static_selector(HITS_SELECTOR.clone(), HITS_SELECTOR_TEXT)?
        )
        .ok_or(Ao3ApiError::SelectorError(
            "Failed to select hits from dd.hits".to_string(),
        ))?
        .trim(),
    );

    Ok(FicMetadata::new(id, name, url, last_updated)
        .with_tags(tags)
        .with_description(desc)
        .with_authors(authors)
        .with_fandom(fandom)
        .with_ship_type(ship_type)
        .with_language(language)
        .with_chapters(chapters)
        .with_kudos(Some(kudos.unwrap_or(0)))
        .with_words(words.ok())
        .with_series(series)
        .with_hits(hits.ok()))
}
