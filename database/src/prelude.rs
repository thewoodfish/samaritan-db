use async_std::sync::Mutex;
use log::{error, info};
use rocket::{
    fairing::{self, Fairing, Info, Kind},
    serde::Deserialize,
    Build, Rocket,
};
use simplelog::{CombinedLogger, Config, LevelFilter, WriteLogger};

use crate::util;

type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;
type GenericResult<T> = Result<T, GenericError>;

lazy_static::lazy_static! {
    static ref LOGGER_INITIALIZED: Mutex<bool> = Mutex::new(false);
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct DbConfig {
    pub application_did: String,
    pub version: String,
}

impl DbConfig {
    // static functions
    pub fn did_is_correct(did: &str) -> bool {
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

#[rocket::async_trait]
impl Fairing for DbConfig {
    fn info(&self) -> Info {
        Info {
            name: "Database Config Checker",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        // setup logger
        match rocket.figment().extract_inner("dblog") {
            Ok(file_addr) => {
                // check that the did is conformant
                match setup_tracing_logger(&file_addr).await {
                    Ok(_) => log::info!("Config: database logging setup complete"),
                    Err(e) => {
                        println!("{:#?}", e);
                        return Err(rocket)},
                }
            }
            Err(_) => return Err(rocket),
        }

        match rocket.figment().extract_inner("application_did") {
            Ok(app_did) => {
                // check that the did is conformant
                if DbConfig::did_is_correct(app_did) {
                    log::info!("Config: application DID found {}", app_did);
                } else {
                    return Err(rocket);
                }
            }
            Err(_) => return Err(rocket),
        }

        Ok(rocket)
    }
}

/// setup async tracing logger
async fn setup_tracing_logger(log_file: &String) -> GenericResult<()> {
    // Check if the logger has already been initialized
    let mut initialized = LOGGER_INITIALIZED.lock().await;
    if *initialized {
        return Ok(());
    }
    // Open the log file in "create or append" mode
    let file = util::create_or_append_file(log_file)?;
    let _ = CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        file,
    )])?;

    // Set the initialized flag to true
    *initialized = true;

    Ok(())
}
