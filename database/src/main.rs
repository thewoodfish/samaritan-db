/// Copyright (c) Algorealm, Inc.

#[macro_use]
extern crate rocket;

mod prelude;
mod routes;

use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment, Profile,
};
use rocket::{fairing::AdHoc, Config};
use prelude::*;

#[launch]
fn rocket() -> _ {
    let figment = Figment::from(rocket::Config::default())
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("Sam.toml").nested())
        .merge(Env::prefixed("SAM_").global())
        .select(Profile::from_env_or("SAM_PROFILE", "default"));

    rocket::custom(figment)
        .mount("/", routes::routes())
        .attach(AdHoc::config::<DbConfig>())
}
