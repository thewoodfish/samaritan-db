/// Copyright (c) Algorealm, Inc.

#[macro_use]
extern crate rocket;

mod contract;
mod db;
mod prelude;
mod routes;
mod util;

use prelude::*;
use rocket::{fairing::AdHoc, http::Header};

#[launch]
fn rocket() -> _ {
    // read config into state
    let path = util::read_config("data", "path");
    let log = util::read_config("data", "log");
    let flush_interval = util::read_config("data", "flush_interval")
        .parse::<u64>()
        .unwrap_or(1000);
    let cache_capacity = util::read_config("data", "path")
        .parse::<u64>()
        .unwrap_or(10_000_000_000);
    let version = util::read_config("data", "version");
    let vsn = version.clone();

    rocket::build()
        .attach(AdHoc::on_response("Response Rewriter", move |_, res| {
            let vsn = vsn.clone();
            Box::pin(async move {
                // add to response header
                res.set_header(Header::new("Server", format!("SamaritanDB v{}", vsn)));
            })
        }))
        .mount("/", routes::routes())
        .manage(DbConfig {
            path,
            log,
            flush_interval,
            cache_capacity,
            version,
        })
        .register("/", catchers![routes::not_found, routes::unauthorized, routes::bad_request])
}
