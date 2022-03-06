use actix_web::{App, HttpServer, web};
use awc::Client;
use clap::Parser;
use tracing_subscriber;

use config::read_toml_file;
use handlers::{info, proxy};
use handlers::info::Info;
use middleware::logging;

mod handlers;
mod middleware;
mod config;
mod utils;

/// The main entry point of the application.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let config_file = read_toml_file(args.config_file).unwrap();

    match config_file.logging.mode {
        config::LoggingMode::Json => tracing_subscriber::fmt()
            .with_level(false)
            .without_time()
            .with_target(false)
            .init(),
        config::LoggingMode::Plain => tracing_subscriber::fmt()
            .with_target(false)
            .init()
    };

    let json = utils::convert(config_file.logging.json.format);

    HttpServer::new(move || {
        App::new()
            // data to share across requests/handlers
            .app_data(web::Data::new(Client::default()))
            .app_data(web::Data::new(utils::to_url(config_file.proxy.address.clone(), config_file.proxy.port)))
            .app_data(web::Data::new(Info::new()))
            // middleware
            .wrap(logging::Logging { json: json.clone() })
            // handlers
            .default_service(web::to(proxy::proxy))
            .service(
                web::scope("/emissary")
                    .route("/info", web::get().to(info::info))
            )
    })
        .bind((config_file.http.address, config_file.http.port))?
        .run()
        .await
}

/// The struct that holds the CLI arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// The path to the configuration file.
    #[clap(short, long, default_value_t = String::from("./config.toml"))]
    pub config_file: String,
}