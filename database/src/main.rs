/// Copyright (c) Algorealm, Inc.

#[macro_use]
extern crate rocket;

mod contract;
mod db;
mod prelude;
mod routes;
mod util;

use std::{sync::Arc, time::Duration};

use async_std::sync::Mutex;
use prelude::*;
use rocket::{fairing::AdHoc, http::Header};

/// Rocket serves as the main entry point to the database.
/// It accepts the HTTP requests and then passes it into other components of the DB
/// Other components are independent of rocket and gets fired up when the database is run e.g networking
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // read config into state
    let path = util::read_config("data", "path");
    let log_ = util::read_config("data", "log");
    let flush_interval = util::read_config("data", "flush_interval")
        .parse::<u64>()
        .unwrap_or(1000);
    let cache_capacity = util::read_config("data", "path")
        .parse::<u64>()
        .unwrap_or(10_000_000_000);
    let version = util::read_config("data", "version");
    let vsn = version.clone();

    // TODO!
    // The default values should not be "empty" but should be set to meaningful defaults

    // set up the config struct
    let config = DbConfig {
        path: path.into(),
        log: log_.into(),
        flush_interval,
        cache_capacity,
        version: version.into(),
    };

    // Queue containing list of DIDs whose data have just been written to the database
    let did_list = Arc::new(Mutex::new(DidQueue::new()));

    // This task runs forever, checking the chain for DIDs validity
    // and taking the necessary actions
    let cfg = config.clone();
    let did_queue = did_list.clone();
    async_std::task::spawn(async move {
        loop {
            // check queue
            let mut guard = did_queue.lock().await;
            if let Some(db_entry) = guard.pop_front() {
                // check the list of DIDs we have recorded and recognized
                if util::read_config("identifiers", &db_entry.did.0).is_empty() {
                    // check the chain if the DID is recognized
                    if !contract::did_exists(&db_entry.did) {
                        // remove data in association to "fake" DID
                        let _ = db::delete_document(&db_entry.db_name, &db_entry.doc_id, &cfg);
                    } else {
                        // write to config file
                        util::write_config("identifiers", &db_entry.did.0, "true");
                    }
                } 
            }

            // sleep for some seconds
            async_std::task::sleep(Duration::from_secs(DID_CLEAUNUP_SLEEP_TIME)).await;
        }
    });

    rocket::build()
        .attach(AdHoc::on_response("Response Rewriter", move |_, res| {
            let vsn = vsn.clone();
            Box::pin(async move {
                // add to response header
                res.set_header(Header::new("Server", format!("SamaritanDB v{}", vsn)));
            })
        }))
        .mount("/", routes::routes())
        // add the did queue as a rocket state, so it can be accessed by internal DB functions
        .manage(did_list)
        .manage(config)
        .register(
            "/",
            catchers![routes::not_found, routes::unauthorized, routes::bad_request],
        )
        .launch()
        .await?;

    Ok(())
}
