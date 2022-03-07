use actix_web::{Responder, Result, web};
use serde::Serialize;

/// Information about the service.
#[derive(Serialize, Clone, Debug)]
pub struct Info {
    pub version: String,
}

impl Info {
    /// Create a new instance of the `Info` struct.
    ///
    /// The version is retrieved from the `Cargo.toml` file.
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Return the information about the service.
pub async fn info(info: web::Data<Info>) -> Result<impl Responder> {
    Ok(web::Json(info))
}