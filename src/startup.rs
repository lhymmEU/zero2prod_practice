use actix_web::{web, App, HttpServer};
use actix_web::dev::Server;
use std::net::TcpListener;
use crate::routes::{ health_check, subscribe };
use sqlx::PgPool;

// start the server and return a Tokio server handler,
// the reason to use listener as an input is,
// we want to run the server on a random port,
// but the port number is not available within the context of this library,
// so we need to pass it into this function
pub fn run(
    listener: TcpListener,
    db_pool: PgPool
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    // this outer block handles the transport layer logic
    let server = HttpServer::new(move || {
        // this app block handles the application layer logic
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}