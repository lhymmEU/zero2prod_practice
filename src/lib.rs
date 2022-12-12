use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_web::dev::Server;
use std::net::TcpListener;

// a dummy function to test if the bare minimum of code works
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

// start the server and return a Tokio server handler,
// the reason to use listener as an input is,
// we want to run the server on a random port,
// but the port number is not available within the context of this library,
// so we need to pass it into this function
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    // this outer block handles the transport layer logic
    let server = HttpServer::new(|| {
        // this app block handles the application layer logic
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    Ok(server)
}