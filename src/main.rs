use std::net::TcpListener;
use zero2prod::startup::run;
use zero2prod::configuration::get_configuration;
use sqlx::PgPool;
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
    // connect to postgres database using a connection string generated from the config file.
    // the reason to use PgPool is: 
    // each time a connection to database is made,
    // PgPool will either create a new connection or,
    // wait for a current connection to close,
    // thus enables concurrent access through multiple connections to a database.
    // (sqlx cannot perform concurrent access over one single connection)
    //
    // the connection string is protected using Secret<String>, need to be exposed before further use
    let connection_pool = PgPool::connect(&configuration.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");
    // get the address for the application server to run on
    let address = format!("127.0.0.1:{}", configuration.application_port);
    // listen to the address
    let listener = TcpListener::bind(address)?;
    // run the server on the address with the previous generated database connection
    run(listener, connection_pool)?.await
}

