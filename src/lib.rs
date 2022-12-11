// To-Do list:
//      1. what is async? : https://www.youtube.com/watch?v=skos4B5x7qE
//      2. what is guard trait?
//      3. what is Futures?: https://cfsamson.github.io/books-futures-explained/introduction.html
//      4. what is procedural macro?
//      5. what is builder-pattern?
//      6. what is user-visible regression?

use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_web::dev::Server;
use std::net::TcpListener;

// a dummy function to test if the bare minimum of code works
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
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
    })
    .listen(listener)?
    .run();

    Ok(server)
}