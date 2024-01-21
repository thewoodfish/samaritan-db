use rocket::http::Status;
use rocket::response::status::{self, Custom};
use rocket::serde::json::{serde_json::json, Json, Value};
use rocket::Request;
use rocket::State;

use crate::{contract, db, prelude::*, util};

#[post("/_auth", data = "<auth_payload>")]
async fn create_user(auth_payload: Json<AuthPayload>) -> status::Custom<Value> {
    let credentials = auth_payload.into_inner();

    // check the DID for lexical compliance
    if DbConfig::is_valid_did(&credentials.did) {
        return Custom(
            Status::InternalServerError,
            json!({
                "error" : format!("DID `{}` is not well formed", credentials.did)
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
                if util::write_config("auth", "application_did", &credentials.did)
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
                            "error" : "could not modify config file"
                        }),
                    );
                }
            } else {
                return Custom(
                    Status::Unauthorized,
                    json!({
                        "error" : format!("DID `{}` already initialized in database", application_did)
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
                    "error": "could not create database"
                }),
            )
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
pub fn delete_db(db_name: &str, auth: BasicAuth, config: &State<DbConfig>) -> (Status, Value) {
    // check if database is in existence
    let config = config.inner();
    if db::database_exists(db_name) {
        // delete the database
        match db::delete_database(auth, config, &db_name) {
            Ok(_) => (
                Status::Created,
                json!({
                    "ok": "true"
                }),
            ),
            Err(_) => (
                Status::InternalServerError,
                json!({
                    "error": "could not create database"
                }),
            )
        }
    } else {
        return (
            Status::NotFound,
            json!({
                "error": "The database does not exist"
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

pub fn routes() -> Vec<rocket::Route> {
    routes![index, create_user, create_db]
}
