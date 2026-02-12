//! Blocking authentication implementation for AO3
use crate::networking::get_page;
use log::debug;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
use std::thread::sleep;
use std::time::Duration;

/// Login information for AO3 authentication
#[derive(Debug, Clone)]
pub struct LoginInfo {
    pub username: Box<str>,
    pub password: Box<str>,
}

/// Authentication token from AO3
#[derive(Deserialize, Debug)]
pub struct Token {
    pub token: String,
}

/// Get login information from text file at provided path
///
/// # Arguments
/// * `path` - Path to login file (username on first line, password on second)
///
/// # Returns
/// * Returns LoginInfo struct with username and password
///
/// # Example
/// ```no_run
/// use ao3_api_rs::networking::get_login_info;
/// let info = get_login_info("log.txt");
/// ```
#[inline(always)]
pub fn get_login_info(path: &str) -> LoginInfo {
    let file = fs::read_to_string(path).expect("failed to read login file");
    let mut lines = file.lines();
    LoginInfo {
        username: lines.next().expect("Username not found").to_owned().into(),
        password: lines.next().expect("Password not found").to_owned().into(),
    }
}

/// Get an auth token for the client's session
///
/// # Arguments
/// * `client` - reqwest Client being used
///
/// # Returns
/// * Returns an auth token as String
///
/// # Example
/// ```no_run
/// use ao3_api_rs::networking::{create_client, get_token};
/// let client = create_client("test").unwrap();
/// let token = get_token(&client);
/// ```
pub fn get_token(client: &Client) -> String {
    let temp = get_page("https://archiveofourown.org/token_dispenser.json", client)
        .unwrap()
        .text()
        .unwrap();
    let j: Token = serde_json::from_str(&temp).unwrap();
    debug!("Token is: {}", j.token);

    j.token.to_string()
}

/// Login to AO3 with credentials from a file
///
/// # Arguments
/// * `client` - reqwest Client with cookie store enabled
/// * `login_file` - Path to login file (username on first line, password on second)
///
/// # Example
/// ```no_run
/// use ao3_api_rs::networking::{create_client, login};
/// let client = create_client("test").unwrap();
/// login(&client, "log.txt");
/// ```
pub fn login(client: &Client, login_file: &str) {
    // get the auth token
    let token = get_token(client);
    sleep(Duration::from_secs(2));
    // we get login information from the file
    let info = get_login_info(login_file);
    // create the request body using format! for better performance
    let loginbody = format!(
        "authenticity_token={}&user%5Blogin%5D={}&user%5Bpassword%5D={}&commit=Log+In",
        token, info.username, info.password
    );
    // set the post request to log in
    let _page = client
        .post("https://archiveofourown.org/users/login")
        .body(loginbody)
        .send()
        .expect("Failed to send login request");
    sleep(Duration::from_secs(2));
    println!("logged in");
}
