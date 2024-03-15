/// Copyright (c) Algorealm, Inc.
use base64::{engine::general_purpose::STANDARD, Engine as _};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    serde::{
        json::{
            serde_json::{self, json},
            Value,
        },
        Deserialize,
    },
    Request,
};
use serde::Serialize;
use serde_json::Error as SerdeError;
use sled::Error as SledError;
use std::{collections::VecDeque, io, time::Duration};

use crate::util;

#[derive(Debug)]
pub enum DatabaseError {
    SerdeError(SerdeError),
    SledError(SledError),
    IoError,
    RevisionIdParseError,
    DocumentUpdateConflict,
    DocumentRevisionNotFound,
    UserDidConflict,
    MissingDocument,
    OtherError,
}

impl From<SerdeError> for DatabaseError {
    fn from(error: SerdeError) -> Self {
        DatabaseError::SerdeError(error)
    }
}

impl From<SledError> for DatabaseError {
    fn from(error: SledError) -> Self {
        DatabaseError::SledError(error)
    }
}

impl From<io::Error> for DatabaseError {
    fn from(_: io::Error) -> Self {
        DatabaseError::IoError
    }
}

#[derive(Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct DbConfig {
    pub path: String,
    pub log: String,
    pub flush_interval: u64,
    pub cache_capacity: u64,
    pub version: String,
    pub contract_address: String,
    pub chain_address: String,
    pub mnemonic: String,
}

/// path to config file
pub static CONFIG_FILE_PATH: &str = "config.ini";
/// time for task to go to sleep during DID validity cleanup
pub const DID_CLEANUP_SLEEP_TIME: u64 = 10;
/// time for the database to sleep before retrying CLI operations
pub const CLI_RETRY_DURATION: Duration = Duration::from_secs(5);
/// Maximum number of retries before giving a negative response
pub const MAX_RETRY_COUNT: u64 = 5;
/// Directory of the smart contract
pub static CONTRACT_DIRECTORY: &str = "../../contract";

pub type DatabaseResult<T> = Result<T, DatabaseError>;

impl DbConfig {
    /// The `user` parameter determines which DID we're trying to parse
    pub fn is_valid_did(did: &String, user: bool) -> bool {
        // Expected format: "did:sam:apps:<48 characters hexadecimal string>"
        let parts: Vec<&str> = did.split(':').collect();

        if parts.len() == 4
            && parts[0] == "did"
            && parts[1] == "sam"
            && parts[2] == if user { "root" } else { "apps" }
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
    pub did: Did,
    pub secret: String,
}

// Define an authentication guard
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BasicAuth {
    type Error = Value;

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
                                    let ss58_address =
                                        ss58_address.split(":").nth(3).unwrap_or_default(); // SS58 address
                                    if username == ss58_address
                                        && password == util::read_config("auth", "auth_secret")
                                    {
                                        return Outcome::Success(BasicAuth {
                                            username: username.to_owned(),
                                            password: password.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
            None => {}
        }

        Outcome::Error((
            Status::Unauthorized,
            json!({
                "error": "Invalid or missing Authorization header"
            }),
        ))
    }
}

// DID type
#[derive(serde::Deserialize, Clone, PartialEq)]
pub struct Did(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Did {
    type Error = Value;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        // Extract or generate the DID from the request
        let did = extract_did_from_request(request);
        // validate did
        if DbConfig::is_valid_did(&did, true) {
            return Outcome::Success(Did(did));
        } else {
            return Outcome::Error((
                Status::BadRequest,
                json!({
                    "error": "Invalid or missing X-DID header"
                }),
            ));
        }
    }
}

// Extract the DID from the header
fn extract_did_from_request(request: &Request<'_>) -> String {
    request
        .headers()
        .get_one("X-DID")
        .unwrap_or_default()
        .to_owned()
}

// A generic wrapper struct that includes the data and an optional "_rev" field
#[derive(Debug, Serialize, Deserialize)]
pub struct DataWrapper<T> {
    pub data: T,
}

/// A type that hold DIDs of data that have just been inserted into the database
pub type DidQueue = VecDeque<DbEntry>;

/// Struct that represents database entries, used to run DID validity cleanup operations
pub struct DbEntry {
    pub did: Did,
    pub db_name: String,
    pub doc_id: String,
}
