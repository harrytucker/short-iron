//! Short Iron is a silly pun about golf and URL shortening.
//!
//! # Overview
//!
//! Actix-Web based API for shortening URLs. Includes functionality such as
//! structured logging, robust error handling, and URL shortening!
//!
//! # Endpoints
//! ## `/shorten`
//! - POST Request
//! - Example: `{"url": "https://google.com"}`
//!
//! ## `/{short_url_id}`
//! - GET Request
//! - Example: `GET https://short.fe/-I7FhYVD1d`
//!
//! ## `/misc/debug`
//! - GET Request
//! - Returns all known URLs and short versions in JSON format
//!
//! # Logging
//!
//! Logging in this project relies on the `tracing` crate. Set the environment
//! variable `RUST_LOG` to change logging levels:
//!
//! ```bash
//! # possible: debug, error, warn, trace, info
//! RUST_LOG=debug ./short-iron
//! ```
//!
//! Short Iron outputs logs in the a JSON format consumable by Bunyan. You can
//! grab this utility from NPM and manipulate logs as follows:
//! ```bash
//! ./short-iron | bunyan -o short
//! ```
use std::collections::HashMap;

use actix_web::{App, HttpServer, web};
use async_std::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::debug;
use tracing_actix_web::TracingLogger;

use handlers::{debugger, redirect, shorten};
use logging::*;

mod handlers;
mod logging;

/// Wraps a `String` type for POST requests to shorten URLs.
#[derive(Serialize, Deserialize, Debug)]
pub struct UrlRequest {
    url: String,
}

/// Aliases for `String` for code clarity.
pub type ShortUrl = String;

/// Aliases for `String` for code clarity.
pub type LongURL = String;

/// Wraps a `Mutex` around a `HashMap` for storing URLs and their shortened
/// variants.
#[derive(Debug)]
pub struct KnownUrls {
    urls: RwLock<HashMap<LongURL, ShortUrl>>,
}

/// Sets up the HttpServer and shared resources.
///
/// Configured here:
/// - Tracing Subscriber (Logging)
/// - KnownUrls shared mutable HashMap
/// - AppFactory and HttpServer
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("short-iron".into(), "info".into());
    init_subscriber(subscriber);

    let known_urls = web::Data::new(KnownUrls {
        urls: RwLock::new(HashMap::new()),
    });
    debug!("Allocated RwLock and HashMap for known URLs");

    HttpServer::new(move || {
        App::new()
            .route("/shorten", web::post().to(shorten))
            .route("/{redirect_id}", web::get().to(redirect))
            .route("/misc/debug", web::get().to(debugger))
            .wrap(TracingLogger)
            .app_data(known_urls.to_owned())
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
