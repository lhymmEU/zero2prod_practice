use std::net::TcpListener;
use zero2prod::startup::run;
use zero2prod::configuration::get_configuration;
use sqlx::{PgPool, postgres::PgPoolOptions};
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use secrecy::ExposeSecret;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // get a tracing subscriber for telemetry data,
    // all sub-routines' default subscriber will be this one if no specific function-level subscriber is provided
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    // read configuration from a yaml config file
    let configuration = get_configuration().expect("Failed to read configuration.");
    
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
    // get the address for the application server to run on
    let address = format!(
        "{}:{}", 
        configuration.application.host, configuration.application.port);
    // listen to the address
    let listener = TcpListener::bind(address)?;
    // run the server on the address with the previous generated database connection
    run(listener, connection_pool)?.await
}

