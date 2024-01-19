use rocket::serde::json::{serde_json::json, Value};
use rocket::State;

use crate::prelude::*;

#[get("/")]
pub fn index(config: &State<DbConfig>) -> Value {
    json!({
        "samaritandb": "Hello Explorer",
        "version": config.version(),
        "vendor": {
            "name": "Algorealm, Inc."
        }
    })
}

pub fn routes() -> Vec<rocket::Route> {
    routes![index]
}