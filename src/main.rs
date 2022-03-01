use actix_web::{App, HttpServer, web};
use awc::Client;

use cli::cli::parse_args;
use config::config::read_toml_file;
use handlers::{info, proxy};
use handlers::info::Info;
use middleware::logging;
use utils::net::to_url;

mod handlers;
mod middleware;
mod config;
mod cli;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = parse_args();
    let config_file = read_toml_file(args.config_file);

    let forward_url = to_url(config_file.proxy.address, config_file.proxy.port);

    HttpServer::new(move || {
        App::new()
            // data to share across requests/handlers
            .app_data(web::Data::new(Client::default()))
            .app_data(web::Data::new(forward_url.clone()))
            .app_data(web::Data::new(Info::new().clone()))
            // middleware
            .wrap(logging::SayHi)
            // handlers
            .default_service(web::to(proxy::forward))
            .service(
                web::scope("/emissary")
                    .route("/info", web::get().to(info::info))
            )
    })
        .bind((config_file.http.address, config_file.http.port))?
        .run()
        .await
}