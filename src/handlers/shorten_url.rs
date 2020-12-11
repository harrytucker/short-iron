use actix_web::{error, Responder, Result, web};
use error::ErrorBadRequest;
use nanoid::nanoid;
use tracing::{debug, error, info};
use url::Url;

use crate::{KnownUrls, UrlRequest};

/// Handles POST requests to shorten URLs.
///
/// Takes a JSON body of a [`UrlRequest`](crate::UrlRequest), i.e.
/// ```json
/// {
///   "url": "https://google.com"
/// }
/// ```
/// Returns a shortened URL or a [`BadRequest`](error::ErrorBadRequest)
pub async fn shorten(
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
            return Err(ErrorBadRequest(e));
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
