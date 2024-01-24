/// Copyright (c) Algorealm, Inc.
use rocket::http::Status;
use rocket::response::status::{self, Custom};
use rocket::serde::json::{serde_json::json, Json, Value};
use rocket::Request;
use rocket::State;

use crate::{contract, db, prelude::*, util};

#[post("/_auth", data = "<auth_payload>")]
async fn init_application(auth_payload: Json<AuthPayload>) -> status::Custom<Value> {
    let credentials = auth_payload.into_inner();

    // check the DID for lexical compliance
    if DbConfig::is_valid_did(&credentials.did.0, false) {
        return Custom(
            Status::InternalServerError,
            json!({
                "error" : format!("DID `{}` is not well formed", credentials.did.0)
            }),
        );
    } else {
        // check that DID and password is recognized onchain
        if contract::authenticate(&credentials) {
            // set the auth details, only if it hasn't been set
            // read config file
            let (secret, application_did) = (
                util::read_config("auth", "secret"),
                util::read_config("auth", "application_did"),
            );
            if secret.is_empty() {
                // write details to config file
                if util::write_config("auth", "application_did", &credentials.did.0)
                    && util::write_config("auth", "secret", &credentials.secret)
                {
                    return Custom(
                        Status::Ok,
                        json!({
                            "ok" : true
                        }),
                    );
                } else {
                    return Custom(
                        Status::InternalServerError,
                        json!({
                            "error" : "Could not modify config file"
                        }),
                    );
                }
            } else {
                return Custom(
                    Status::Unauthorized,
                    json!({
                        "error" : format!("DID `{}` already initialized in database.", application_did)
                    }),
                );
            }
        } else {
            return Custom(
                Status::NotFound,
                json!({
                    "error" : "provided details not registered onchain"
                }),
            );
        }
    }
}

#[get("/")]
pub fn index(config: &State<DbConfig>) -> Value {
    json!({
        "samaritandb": "Hello Explorer",
        "version": config.version,
        "vendor": {
            "name": "Algorealm, Inc."
        },
        "application_did": util::read_config("auth", "application_did")
    })
}

/// create a database
#[put("/<db_name>")]
pub fn create_db(db_name: &str, _auth: BasicAuth, config: &State<DbConfig>) -> (Status, Value) {
    // check if database is in existence
    let config = config.inner();
    if !db::database_exists(db_name) {
        // create new database
        match db::create_database(config, &db_name) {
            Ok(_) => (
                Status::Created,
                json!({
                    "ok": "true"
                }),
            ),
            Err(_) => (
                Status::InternalServerError,
                json!({
                    "error": "Could not create database."
                }),
            ),
        }
    } else {
        return (
            Status::Conflict,
            json!({
                "error": "The database has already been created"
            }),
        );
    }
}

/// delete a database
#[delete("/<db_name>")]
pub fn delete_db(db_name: &str, config: &State<DbConfig>, _auth: BasicAuth) -> (Status, Value) {
    // check if database is in existence
    let config = config.inner();
    if db::database_exists(db_name) {
        // delete the database
        match db::delete_database(config, &db_name) {
            Ok(_) => (
                Status::Ok,
                json!({
                    "ok": "true"
                }),
            ),
            Err(_) => (
                Status::InternalServerError,
                json!({
                    "error": "Could not delete database."
                }),
            ),
        }
    } else {
        return (
            Status::NotFound,
            json!({
                "error": "The database does not exist."
            }),
        );
    }
}

/// retrieve a list of all databases
#[get("/_all_dbs")]
pub fn all_dbs(config: &State<DbConfig>) -> (Status, Value) {
    match db::all_dbs(config) {
        Ok(dbs) => (Status::Ok, json!(dbs)),
        Err(_) => (
            Status::InternalServerError,
            json!({
                "error": "Could not fetch databases."
            }),
        ),
    }
}

/// generate arbitrary random UUIDs
#[get("/_uuids?<count>")]
pub fn uuids(count: Option<u32>) -> Value {
    let count = count.unwrap_or(1); // Set default count to 1 if not specified
    let mut uuids: Vec<String> = Vec::with_capacity(37 * count as usize);

    for _ in 0..count {
        let id = util::generate_uuid().to_string();
        uuids.push(id);
    }

    json!(uuids)
}

/// write data
#[put("/<db_name>/<doc_id>", data = "<data_wrapper>")]
pub fn update_document(
    db_name: &str,
    doc_id: &str,
    did: Did,
    config: &State<DbConfig>,
    _auth: BasicAuth,
    data_wrapper: Json<DataWrapper<Value>>,
) -> (Status, Value) {
    // check if database is in existence
    let config = config.inner();
    let data = data_wrapper.into_inner();
    if db::database_exists(db_name) {
        // write to it
        match db::update_document(db_name, doc_id, did, config, data) {
            Ok(json) => (Status::Ok, json),
            Err(e) => match e {
                DatabaseError::DocumentUpdateConflict => (
                    Status::Conflict,
                    json!({
                        "error": "Document update conflict."
                    }),
                ),
                DatabaseError::UserDidConflict => (
                    Status::Conflict,
                    json!({
                        "error": "User DID conflict"
                    }),
                ),
                _ => (
                    Status::InternalServerError,
                    json!({
                        "error": "Could not update database."
                    }),
                ),
            },
        }
    } else {
        return (
            Status::NotFound,
            json!({
                "error": "The database does not exist."
            }),
        );
    }
}

/// read data
#[get("/<db_name>/<doc_id>")]
pub fn fetch_document(db_name: &str, doc_id: &str, config: &State<DbConfig>) -> (Status, Value) {
    // check if database is in existence
    let config = config.inner();
    if db::database_exists(db_name) {
        // fetch document
        match db::fetch_document(db_name, doc_id, config) {
            Ok(json) => (Status::Ok, json),
            Err(e) => match e {
                DatabaseError::MissingDocument => (
                    Status::NotFound,
                    json!({
                        "error": "The document does not exist"
                    }),
                ),
                _ => (
                    Status::InternalServerError,
                    json!({
                        "error": "Could not read from database."
                    }),
                ),
            },
        }
    } else {
        return (
            Status::NotFound,
            json!({
                "error": "The database does not exist."
            }),
        );
    }
}

/// get document metadata
#[delete("/<db_name>/<doc_id>")]
pub fn delete_document(db_name: &str, doc_id: &str, config: &State<DbConfig>) -> (Status, Value) {
    // check if database is in existence
    let config = config.inner();
    if db::database_exists(db_name) {
        // fetch document
        match db::delete_document(db_name, doc_id, config) {
            Ok(_) => (Status::Ok, json!({ "ok": true})),
            Err(_) => (
                Status::InternalServerError,
                json!({
                    "error": "Could not delete document"
                }),
            ),
        }
    } else {
        return (
            Status::NotFound,
            json!({
                "error": "The database does not exist."
            }),
        );
    }
}

#[catch(404)]
pub fn not_found(req: &Request) -> Value {
    json!({
        "error": format!("`{}` route not recognized", req.uri())
    })
}

#[catch(401)]
pub fn unauthorized(_req: &Request) -> Value {
    json!({
        "error": "authentication details is incorrect"
    })
}

#[catch(400)]
pub fn bad_request(_req: &Request) -> Value {
    json!({
        "error": "Invalid or missing important header"
    })
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        index,
        init_application,
        create_db,
        delete_db,
        all_dbs,
        uuids,
        update_document,
        fetch_document,
        delete_document
    ]
}
