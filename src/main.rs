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
mod logging;

use actix_web::{error, web, App, HttpServer, Responder, Result};
use async_std::sync::RwLock;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

// logging imports
use logging::*;
use tracing::{debug, error, info};
use tracing_actix_web::TracingLogger;

/// Wraps a `String` type for POST requests to shorten URLs.
#[derive(Serialize, Deserialize, Debug)]
struct UrlRequest {
    url: String,
}

/// Aliases for `String` for code clarity.
type ShortUrl = String;

/// Aliases for `String` for code clarity.
type LongURL = String;

/// Wraps a `Mutex` around a `HashMap` for storing URLs and their shortened
/// variants.
#[derive(Debug)]
struct KnownUrls {
    urls: RwLock<HashMap<LongURL, ShortUrl>>,
}

/// Handles POST requests to shorten URLs.
///
/// Takes a JSON body of a [`UrlRequest`](crate::UrlRequest), i.e.
/// ```json
/// {
///   "url": "https://google.com"
/// }
/// ```
/// Returns a shortened URL or a [`BadRequest`](error::ErrorBadRequest)
async fn shorten(
    url_req: web::Json<UrlRequest>,
    known_urls: web::Data<KnownUrls>,
) -> Result<String> {
    let submitted_url = &url_req.url.to_string();
    let valid_url = match Url::parse(submitted_url) {
        Ok(url) => {
            debug!("Submitted url is valid");
            url
        }
        Err(e) => {
            error!(
                input = submitted_url.as_str(),
                "Failed to parse input as a URL",
            );
            return Err(error::ErrorBadRequest(e));
        }
    };

    let mut urls = known_urls.urls.write().await;
    debug!(?urls, "Obtained write lock to known URLs");

    // if the URL exists as a key, then return the already generated short URL,
    // otherwise generate a new ID and short URL, then send the response.
    match urls.get(&valid_url.to_string()) {
        Some(existing) => {
            debug!(
                shortened_url = ?existing,
                "Submitted URL already shortened."
            );
            Ok(existing.into())
        }
        None => {
            debug!(
                url = ?submitted_url,
                "URL not yet recorded, generating ID"
            );
            let shortened = format!("short.fe/{}", nanoid!(10));

            // it's the first time this value is inserted, so HashMap.insert()
            // will return a `None` variant that we'll throw out
            match urls.insert(valid_url.to_string(), shortened.to_string()) {
                None => {}
                _ => {}
            }

            info!(
                shortened_url = shortened.as_str(),
                "Generated shortened URL"
            );
            Ok(shortened.into())
        }
    }
}

/// Redirects requests from shortened URLs to their expanded version
///
/// Returns either a 303 See Other response, or a 404 Not Found.
async fn redirect(
    redirect_id: web::Path<String>,
    known_urls: web::Data<KnownUrls>,
) -> impl Responder {
    let urls = known_urls.urls.read().await;
    debug!(?urls, "Obtained read lock to known URLs");

    let short_url = format!("short.fe/{}", redirect_id.0.to_string());
    debug!(
        constructed_url = ?short_url,
        "Constructed full URL from request"
    );

    // finds the first matching URL in the HashMap
    let expanded_url = urls.iter().find_map(|(key, val)| {
        if val.to_string() == short_url {
            debug!(
                expanded_url = ?key,
                "Expanded short URL to full URL"
            );
            Some(key)
        } else {
            None
        }
    });

    // checks the option found by expanded_url, if a URL is present then return
    // a 303 See Other response for the expanded URL
    match expanded_url {
        Some(url) => {
            info!(?expanded_url, "Redirected to expanded URL");
            return web::HttpResponse::SeeOther()
                .header("Location", url.to_string())
                .await;
        }
        // else, return 404 Not Found
        None => {
            info!(?short_url, "Short URL isn't registered, no redirect");
            return web::HttpResponse::NotFound().await;
        }
    }
}

/// Responds with a JSON representation of the HashMap of known URLs for
/// debugging purposes
async fn debugger(known_urls: web::Data<KnownUrls>) -> impl Responder {
    let urls = known_urls.urls.read().await;

    // this handler needs to return the HashMap, and not the RwLockReadGuard
    web::Json(urls.to_owned())
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
