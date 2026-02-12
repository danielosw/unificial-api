//! Blocking HTTP client implementation for AO3

use crate::utils::{arcify, make_selector};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::{self, redirect};
use scraper::{ElementRef, Html};
use std::env::current_dir;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;
use std::{fs, thread::sleep, time};

/// Compiled regex for extracting page numbers (compiled once at first use)
static PAGE_NUM_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)(\d*)$").expect("Failed to create page number regex"));

/// Create a configured HTTP client for AO3 operations
///
/// # Returns
/// * Returns a configured reqwest Client
///
/// # Example
/// ```no_run
/// use ao3_api_rs::networking::create_client;
/// let client = create_client("test").expect("Failed to create client");
/// ```
pub fn create_client(useragent: &str) -> Result<Client, reqwest::Error> {
    Client::builder()
        .redirect(redirect::Policy::none())
        .cookie_store(true)
        .timeout(Duration::new(960, 0))
        .user_agent(useragent)
        .build()
}

/// Get the requested URL with the provided client
///
/// # Arguments
/// * `url` - URL to fetch
/// * `client` - reqwest Client to use
///
/// # Returns
/// * Returns a Result with the Response or an error
///
/// # Example
/// ```no_run
/// use ao3_api_rs::networking::{create_client, get_page};
/// let client = create_client("test").unwrap();
/// let response = get_page("https://archiveofourown.org", &client);
/// ```
pub fn get_page(url: &str, client: &Client) -> Result<reqwest::blocking::Response, reqwest::Error> {
    println!("Did request to {}", url);
    let response = client.get(url).send().expect("Get request failed");
    println!("{}", response.status());

    match response.status() {
        // handle redirect
        status
            if (status == reqwest::StatusCode::FOUND
                || status == reqwest::StatusCode::MOVED_PERMANENTLY)
                && !url.contains("login") =>
        {
            // get the redirect location
            let i = response
                .headers()
                .get("location")
                .expect("Getting location value for redirect failed")
                .to_str()
                .expect("Failed to convert location header to string");
            // TODO: check for infinite redirect loops
            println!("Following redirect");
            sleep(time::Duration::from_secs(2));
            let redirect_url = if i.starts_with("http") {
                i.to_string()
            } else {
                format!("https://archiveofourown.org{}", i)
            };
            get_page(&redirect_url, client)
        }
        // handle timeout
        status if matches!(status.as_u16(), 503 | 408 | 429 | 525 | 502 | 524) => {
            // 503 debug
            let writeto = format!(
                "{}/output/",
                current_dir()
                    .expect("Failed to get current directory")
                    .display()
            );
            // set default retry time
            let mut retrytime = 20;
            // try to set retrytime to requested timeout
            if let Some(retry_header) = response.headers().get("Retry_After") {
                retrytime = retry_header
                    .to_str()
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(20);
            }
            // write debug file
            if let Ok(text) = response.text() {
                let _ = fs::write(format!("{}debug.html", writeto), text);
            }

            sleep(time::Duration::from_secs(retrytime));

            println!("Service Unavailable, Retrying");

            get_page(url, client)
        }
        reqwest::StatusCode::OK => {
            sleep(time::Duration::from_secs(5));
            Ok(response)
        }
        status => {
            // I don't want to be blindly doing things when I don't know what we are supposed to do so we just panic.
            panic!("Unknown status: {}", status);
        }
    }
}

/// Get the initial page and aggregate multiple pages if pagination exists
///
/// # Arguments
/// * `page` - URL of the page to fetch
/// * `client` - reqwest Client to use
///
/// # Returns
/// * Returns parsed HTML with all pages aggregated
///
/// # Example
/// ```no_run
/// use ao3_api_rs::networking::{create_client, get_init_page};
/// let client = create_client("test").unwrap();
/// let html = get_init_page("https://archiveofourown.org/works".to_string(), &client);
/// ```
pub fn get_init_page(page: String, client: &Client) -> Html {
    let ficpage = get_page(&page, client);
    let page1 = Html::parse_document(
        &(ficpage
            .expect("Failed to get fic page")
            .text()
            .expect("failed to get fic page text")),
    );
    // Check if their is more then one page
    let selector = make_selector(r#"ol[class="pagination actions"]"#)
        .expect("failed to make selector for pages");
    let mut nav = page1.select(&selector);
    // We just grab item one
    let atags = make_selector(r#"a"#).expect("failed to make selector for atags");
    let mut finalpage = page1.html();
    // Handle if their is no nav bar
    if nav.clone().count() != 0 {
        let page = nav.next().expect("failed to get navbar").select(&atags);
        let vec: Vec<String> = page
            .filter_map(|item: ElementRef<'_>| -> Option<String> {
                let partitle = item
                    .parent()
                    .expect("failed to get node parent")
                    .value()
                    .as_element()
                    .expect("failed to convert a tag node to element")
                    .attr("title");
                if partitle.unwrap_or("LOL") != "next" {
                    Some(
                        item.value()
                            .attr("href")
                            .expect("Failed to get atag href")
                            .to_string(),
                    )
                } else {
                    None
                }
            })
            .collect();

        let pager = vec.last().expect("failed to get final page in vec");

        // Use a regex to extract the page numbers
        let lastpage = PAGE_NUM_REGEX.captures_iter(pager).next();
        let finalvec: Arc<Mutex<Vec<String>>> = arcify(Vec::new());
        let numlist = 1..lastpage.unwrap().get(1).unwrap().as_str().parse().unwrap();

        numlist.into_par_iter().for_each(|i| {
            finalvec.lock().expect("Failed to lock").push(
                PAGE_NUM_REGEX
                    .replace_all(pager, i.to_string())
                    .into_owned(),
            )
        });

        for i in finalvec.lock().expect("Failed to lock").iter() {
            let url = format!("https://archiveofourown.org/{}", i);
            finalpage.push_str(
                &get_page(&url, client)
                    .expect("Getting page failed")
                    .text()
                    .expect("Converting page to text failed"),
            );
        }
    }
    Html::parse_document(&finalpage)
}
