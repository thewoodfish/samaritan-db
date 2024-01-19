/// Copyright (c) Algorealm, Inc.

#[macro_use]
extern crate rocket;

mod prelude;
mod routes;
mod util;

use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment, Profile,
};
use prelude::*;
use rocket::{fairing::AdHoc, Config};

#[launch]
fn rocket() -> _ {
    let figment = Figment::from(rocket::Config::default())
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("Sam.toml").nested())
        .merge(Env::prefixed("SAM_").global())
        .select(Profile::from_env_or("SAM_PROFILE", "default"));

    // extract needed config
    let application_did = figment.extract_inner("application_did").unwrap_or_default();
    let version = figment.extract_inner("version").unwrap_or_default();

    rocket::custom(figment)
        .mount("/", routes::routes())
        .attach(DbConfig {
            application_did,
            version,
        })
        .attach(AdHoc::config::<DbConfig>())
}
