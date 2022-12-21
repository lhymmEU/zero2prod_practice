use actix_web::{web, App, HttpServer};
use actix_web::dev::Server;
use std::net::TcpListener;
use crate::routes::{ health_check, subscribe };
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use crate::email_client::EmailClient;
use crate::configuration::{Settings, DatabaseSettings};
use sqlx::postgres::PgPoolOptions;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
    
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout
        );
    
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, connection_pool, email_client)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}



pub fn get_connection_pool(
    configuration: &DatabaseSettings
) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

// start the server and return a Tokio server handler,
// the reason to use listener as an input is,
// we want to run the server on a random port,
// but the port number is not available within the context of this library,
// so we need to pass it into this function
pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    // wrap the db connection with actix_web's data extractor.
    // the reason is:
    // actix web will spawn an App on each cpu core,
    // and we'd like to share the db connection among these instances,
    // using the data extractor provided by actix_web,
    // we can share the connection with Arc pointer ability,
    // so the concurrent access to the data can be secured.
    let db_pool = web::Data::new(db_pool);
    
    let email_client = web::Data::new(email_client);
    
    // this outer block handles the transport layer logic
    let server = HttpServer::new(move || {
        // this app block handles the application layer logic
        App::new()
            // TracingLogger is a replacement for actix_web's native logger,
            // it provides an easy way to parse actix_web's loggings and
            // integrate them with tracing spans
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}