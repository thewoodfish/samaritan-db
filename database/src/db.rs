use std::io;

use crate::{prelude::*, util};
use rocket::{serde::json::serde_json::json};

/// check if a database exists
pub fn database_exists(db_name: &str) -> bool {
    let data_path = util::read_config("data", "path");
    let db_path = format!("{}{}", data_path, db_name);

    util::is_directory_within_parent(&db_path, &data_path)
}

/// create a database
pub fn create_database(config: &DbConfig, name: &str) -> GenericResult<()> {
    let cfg = sled::Config::default()
        .path(format!("{}{}", config.path, name))
        .cache_capacity(config.cache_capacity)
        .flush_every_ms(Some(config.flush_interval));

    // create database
    let _ = cfg.open()?;

    // record metadata in the .dbs database
    let cfg = cfg.path(format!("{}.dbs", config.path));
    let root_db = cfg.open()?;

    let db_meta = json!({
        "id": util::generate_uuid().to_string(),
        "application_did": util::read_config("auth", "application_did"),
        "created_at": util::get_unix_epoch_time(),
    })
    .to_string();

    // insert into root db
    root_db.insert(name.as_bytes(), db_meta.as_bytes())?;

    Ok(())
}

/// delete a database
/// This majorly entails removing the db directory and clearing its metadata
pub fn delete_database(auth: BasicAuth, config: &DbConfig, name: &str) -> GenericResult<()> {
    // cross check that the authenticated party is the db creator

    // get metadata entry
    let cfg = sled::Config::default()
        .path(format!("{}{}", config.path, name))
        .cache_capacity(config.cache_capacity)
        .flush_every_ms(Some(config.flush_interval));

    let cfg = cfg.path(format!("{}.dbs", config.path));
    let root_db = cfg.open()?;

    // perform removal
    let entry = root_db.get(name.as_bytes())?.ok_or(io::Error::new(io::ErrorKind::Other, "Irrelevant error"))?;
    // convert to string
    let metadata = String::from_utf8(entry.to_vec())?;

    // delete directory
    Ok(())
}