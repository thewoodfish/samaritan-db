/// Copyright (c) Algorealm, Inc.
use std::fs;

use crate::{prelude::*, util};
use rocket::serde::json::{
    serde_json::{self, from_str, json},
    Json, Value,
};

/// check if a database exists
pub fn database_exists(db_name: &str) -> bool {
    let data_path = util::read_config("data", "path");
    let db_path = format!("{}{}", data_path, db_name);

    util::is_directory_within_parent(&db_path, &data_path)
}

/// create a database
pub fn create_database(config: &DbConfig, name: &str) -> Result<(), DatabaseError> {
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
pub fn delete_database(config: &DbConfig, name: &str) -> Result<(), DatabaseError> {
    // get metadata entry
    let cfg = sled::Config::default()
        .path(format!("{}.dbs", config.path))
        .cache_capacity(config.cache_capacity)
        .flush_every_ms(Some(config.flush_interval));

    let root_db = cfg.open()?;

    // remove metadata
    match root_db.remove(name.as_bytes())? {
        Some(_) => {
            // finally, remove directory
            let data_path = util::read_config("data", "path");
            let db_path = format!("{}{}", data_path, name);

            fs::remove_dir_all(db_path)?;
        }
        None => {
            return Err(DatabaseError::OtherError);
        }
    };

    Ok(())
}

/// Get a list of all the databases
pub fn all_dbs(config: &DbConfig) -> DatabaseResult<Vec<String>> {
    // get metadata entry
    let cfg = sled::Config::default()
        .path(format!("{}.dbs", config.path))
        .cache_capacity(config.cache_capacity)
        .flush_every_ms(Some(config.flush_interval));

    let root_db = cfg.open()?;

    Ok(root_db
        .iter()
        .filter_map(Result::ok)
        .map(|(key, _)| String::from_utf8_lossy(&key).to_string())
        .collect::<Vec<String>>())
}

/// write to database
pub fn update_document(
    db_name: &str,
    doc_id: &str,
    did: Did,
    config: &DbConfig,
    data_wrapper: &Json<DataWrapper<String>>,
) -> Result<Value, DatabaseError> {
    // first parse the data wrapper
    let mut db_entry: Value = serde_json::from_str(&data_wrapper.data)?;

    let cfg = sled::Config::default()
        .path(format!("{}{}", config.path, db_name))
        .cache_capacity(config.cache_capacity)
        .flush_every_ms(Some(config.flush_interval));

    let db = cfg.open()?;

    // metadata ID = doc_id + "_meta"
    let meta_id = format!("{}_meta", doc_id);

    // _rev signifies an update
    if let Some(rev) = &data_wrapper._rev {
        // get the document entry and its metadata
        let doc = db
            .get(doc_id.as_bytes())?
            .ok_or(DatabaseError::OtherError)?;

        let doc_meta = db
            .get(doc_id.as_bytes())?
            .ok_or(DatabaseError::OtherError)?;

        let doc = from_str::<Value>(
            &String::from_utf8(doc.to_vec())
                .ok()
                .ok_or(DatabaseError::OtherError)?,
        )?;
        let mut doc_meta = from_str::<Value>(
            &String::from_utf8(doc_meta.to_vec())
                .ok()
                .ok_or(DatabaseError::OtherError)?,
        )?;

        // extract _rev_id and compare
        if *rev == doc["_rev"].to_string() {
            // check for did correlation
            if did.0 == doc_meta["_did"].to_string() {
                // update data
                let new_entry = util::merge_json_values(doc, db_entry);
                let (mut new_entry, current_rev) = util::remove_field(new_entry, "_rev");
                if let Some(_rev) = current_rev {
                    // get new rev
                    let _rev = _rev.to_string();
                    let parsed_rev = _rev
                        .split("-")
                        .next()
                        .ok_or(DatabaseError::RevisionIdParseError)?;

                    let parsed_rev = parsed_rev
                        .parse::<u64>()
                        .ok()
                        .ok_or(DatabaseError::OtherError)?;
                    let new_rev = util::generate_rev(parsed_rev + 1, &new_entry.to_string());

                    // set rev
                    new_entry["_rev"] = new_rev.clone().into();

                    // save new document
                    db.insert(doc_id.as_bytes(), new_entry.to_string().as_bytes())?;

                    // update document metadata
                    doc_meta["_rev"] = new_rev.clone().into();
                    doc_meta["updated_at"] = util::get_unix_epoch_time().into();

                    // save metadata
                    db.insert(meta_id.as_bytes(), doc_meta.to_string().as_bytes())?;

                    // return response
                    return Ok(json!({
                        "ok": true,
                        "id": doc_id,
                        "rev": new_rev
                    }));
                } else {
                    // error [should never happen under normal circumstances]
                    return Err(DatabaseError::DocumentRevisionNotFound);
                }
            } else {
                return Err(DatabaseError::UserDidConflict);
            }
        } else {
            // we can't do anything
            // client is not referring to this copy of data in time
            // most important error to note
            return Err(DatabaseError::DocumentUpdateConflict);
        }
    } else {
        // create new document entry in the database
        // update id
        db_entry["id"] = doc_id.to_owned().into();
        // update rev
        let rev = util::generate_rev(1, &db_entry.to_string());
        db_entry["_rev"] = rev.clone().into();

        // save entry
        db.insert(doc_id.as_bytes(), db_entry.to_string().as_bytes())?;

        // save the document metadata too
        let metadata = json!({
            // accessible by default, except changed in contract
            "_accessible": true,
            "_did": did.0,
            "_rev": rev.clone(),
            "created_at": util::get_unix_epoch_time(),
            "updated_at": util::get_unix_epoch_time(),
        })
        .to_string();

        // save in same database
        db.insert(meta_id.as_bytes(), metadata.to_string().as_bytes())?;

        // return response
        return Ok(json!({
            "ok": true,
            "id": did.0,
            "rev": rev
        }));
    }
}

/// read from database
pub fn fetch_document(db_name: &str, doc_id: &str, config: &DbConfig) -> DatabaseResult<Value> {
    // open database
    let cfg = sled::Config::default()
        .path(format!("{}{}", config.path, db_name))
        .cache_capacity(config.cache_capacity)
        .flush_every_ms(Some(config.flush_interval));

    let db = cfg.open()?;

    let document = db
        .get(doc_id.as_bytes())?
        .ok_or(DatabaseError::OtherError)?;

    let doc_string = String::from_utf8(document.to_vec())
        .ok()
        .ok_or(DatabaseError::OtherError)?;

    Ok(json!(from_str::<Value>(&doc_string)?))
}
