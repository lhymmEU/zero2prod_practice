use std::net::TcpListener;
use zero2prod::startup::run;
use zero2prod::configuration::get_configuration;
use sqlx::PgPool;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new(
        "zero2prod".into(),
        std::io::stdout
    );
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");
    // read configuration from a yaml config file
    let configuration = get_configuration().expect("Failed to read configuration.");
    // connect to postgres database using a connection string generated from the config file.
    // the reason to use PgPool is: 
    // each time a connection to database is made,
    // PgPool will either create a new connection or,
    // wait for a current connection to close,
    // thus enables concurrent access through multiple connections to a database.
    // (sqlx cannot perform concurrent access over one single connection)
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    // get the address for the application server to run on
    let address = format!("127.0.0.1:{}", configuration.application_port);
    // listen to the address
    let listener = TcpListener::bind(address)?;
    // run the server on the address with the previous generated database connection
    run(listener, connection_pool)?.await
}