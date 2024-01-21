use rocket::{
    serde::Deserialize, request::{FromRequest, Outcome}, http::Status,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};

use crate::util;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct DbConfig {
    pub path: String,
    pub log: String,
    pub flush_interval: u64,
    pub cache_capacity: u64,
    pub version: String,
}

/// path to config file
pub const CONFIG_FILE_PATH: &str = "config.ini";

type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type GenericResult<T> = Result<T, GenericError>;

impl DbConfig {
    // static functions
    pub fn is_valid_did(did: &String) -> bool {
        // Expected format: "did:sam:apps:<48 characters hexadecimal string>"
        let parts: Vec<&str> = did.split(':').collect();

        if parts.len() == 4
            && parts[0] == "did"
            && parts[1] == "sam"
            && parts[2] == "apps"
            && parts[3].len() == 48
            && parts[3].chars().all(|c| c.is_ascii_hexdigit())
        {
            true
        } else {
            false
        }
    }
}

/// Authentication payload for assigning an application control of the database
#[derive(serde::Deserialize)]
pub struct AuthPayload {
    pub did: String,
    pub secret: String,
}

// Define an authentication guard
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BasicAuth {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("Authorization") {
            Some(header) => {
                if let Some(credentials) = header.strip_prefix("Basic ") {
                    if let Ok(decoded) = STANDARD.decode(credentials) {
                        if let Ok(credentials_str) = String::from_utf8(decoded) {
                            let mut parts = credentials_str.splitn(2, ':');
                            let username = parts.next();
                            let password = parts.next();

                            if let Some(username) = username {
                                if let Some(password) = password {
                                    // check for equility
                                    let ss58_address = util::read_config("auth", "application_did");
                                    let ss58_address = ss58_address.split(":").nth(3).unwrap_or_default();   // SS58 address
                                    if username == ss58_address && password == util::read_config("auth", "secret") {
                                        return Outcome::Success(BasicAuth { username: username.to_owned(), password: password.to_string() });
                                    }
                                }
                            }
                        }
                    }
                }
            }
            None => {}
        }

        Outcome::Error((Status::Unauthorized, ()))
    }
}