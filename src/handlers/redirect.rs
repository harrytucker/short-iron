use actix_web::{Responder, web};
use tracing::{debug, info};

use crate::KnownUrls;

/// Redirects requests from shortened URLs to their expanded version
///
/// Returns either a 303 See Other response, or a 404 Not Found.
pub async fn redirect(
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
    return match expanded_url {
        Some(url) => {
            info!(?expanded_url, "Redirected to expanded URL");
            web::HttpResponse::SeeOther()
                .header("Location", url.to_string())
                .await
        }
        // else, return 404 Not Found
        None => {
            info!(?short_url, "Short URL isn't registered, no redirect");
            web::HttpResponse::NotFound().await
        }
    }
}
