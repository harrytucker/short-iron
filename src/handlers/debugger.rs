use actix_web::{error, Responder, Result, web};
use tracing::{debug, error, info};
use web::Json;

use crate::KnownUrls;

/// Responds with a JSON representation of the HashMap of known URLs for
/// debugging purposes
pub async fn debugger(known_urls: web::Data<KnownUrls>) -> impl Responder {
    let urls = known_urls.urls.read().await;

    // this handler needs to return the HashMap, and not the RwLockReadGuard
    Json(urls.to_owned())
}
