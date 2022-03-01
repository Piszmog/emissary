use actix_web::{Responder, Result, web};
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct Info {
    pub version: String,
}

impl Info {
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

pub async fn info(info: web::Data<Info>) -> Result<impl Responder> {
    Ok(web::Json(info))
}