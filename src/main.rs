use actix_web::{App, HttpServer, web};
use awc::Client;
use clap::Parser;
use tracing::info;

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
    // get the configuration
    let args = Args::parse();
    let config_file = read_toml_file(args.config_file).unwrap();

    let proxy_url = utils::to_url(config_file.proxy.address, config_file.proxy.port).unwrap();

    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());

    // determine the logging mode
    match config_file.logging.mode {
        config::LoggingMode::Json => tracing_subscriber::fmt()
            .with_level(false)
            .without_time()
            .with_target(false)
            .with_writer(non_blocking)
            .init(),
        config::LoggingMode::Plain => tracing_subscriber::fmt()
            .with_target(false)
            .with_writer(non_blocking)
            .init()
    };

    let json = utils::to_json(config_file.logging.json.format);
    let plain = utils::Plain::from(config_file.logging.plain.format);

    info!("Starting emissary");

    HttpServer::new(move || {
        App::new()
            // HTTP client for the reverse proxy
            .app_data(web::Data::new(Client::default()))
            // URL to the main app
            .app_data(web::Data::new(proxy_url.clone()))
            // information about the app
            .app_data(web::Data::new(Info::new()))
            // logging middleware
            .wrap(logging::Logging {
                mode: config_file.logging.mode.clone(),
                json: json.clone(),
                plain: plain.clone(),
            })
            // all non-mapped routes go to the reverse proxy
            .default_service(web::to(proxy::proxy))
            // emissary services
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
struct Args {
    /// The path to the configuration file.
    #[clap(short, long, default_value_t = String::from("./emissary.toml"))]
    config_file: String,
}
