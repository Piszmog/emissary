/// A simple Web application to test the reverse proxy functionality of emissary.

use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, http::header::ContentType};

#[get("/get")]
async fn hello() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body("Hello world!")
}

#[post("/post")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
    })
        .bind(("127.0.0.1", 8082))?
        .run()
        .await
}