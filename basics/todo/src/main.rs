#[macro_use]
extern crate log;

use std::{env, io};

use actix_files as fs;
use actix_web::cookie::Key;
use actix_web::middleware::{ErrorHandlers, Logger};
use actix_web::{http, web, App, HttpServer};
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web_flash_messages::FlashMessagesFramework;
use dotenv::dotenv;
use tera::Tera;

mod api;
mod db;
mod model;

static SESSION_SIGNING_KEY: &[u8] = &[0; 64];

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    env::set_var("RUST_LOG", "actix_todo=debug,actix_web=info");
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::initiate_connection_pool(&database_url)
        .await
        .expect("Failed to establish database connection");

    let app = move || {
        debug!("Constructing the App");

        let mut templates = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        templates.autoescape_on(vec!["tera"]);

        let message_store =
            CookieMessageStore::builder(Key::from(SESSION_SIGNING_KEY)).build();
        let message_framework = FlashMessagesFramework::builder(message_store).build();

        let error_handlers = ErrorHandlers::new()
            .handler(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                api::internal_server_error,
            )
            .handler(http::StatusCode::BAD_REQUEST, api::bad_request)
            .handler(http::StatusCode::NOT_FOUND, api::not_found);

        App::new()
            .app_data(web::Data::new(templates))
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .wrap(message_framework)
            .wrap(error_handlers)
            .service(web::resource("/").route(web::get().to(api::index)))
            .service(web::resource("/todo").route(web::post().to(api::create)))
            .service(web::resource("/todo/{id}").route(web::post().to(api::update)))
            .service(fs::Files::new("/static", "static/"))
    };

    debug!("Starting server");
    HttpServer::new(app).bind("localhost:8088")?.run().await
}
