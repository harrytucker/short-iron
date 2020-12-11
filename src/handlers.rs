mod debugger;
mod redirect;
mod shorten_url;

// re-export the handlers here to avoid repetitive 'use' statements:
pub use debugger::debugger;
pub use redirect::redirect;
pub use shorten_url::shorten;
