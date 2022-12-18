use std::net::TcpListener;
use sqlx::{ PgPool, PgConnection, Executor, Connection };
use uuid::Uuid;
use zero2prod::configuration::{ get_configuration, DatabaseSettings };
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;

// we need this because the lazy execution will
// only execute the wrapped code once so
// we'll only initialize the subscriber once for one testing run
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // the return type depends on the 3rd (sink) variable
    // since we have two different types of sinks
    // we can't use a general var as an input
    // otherwise the return type cannot be decided by the compiler
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// this function handles the logic of spawn a server to the background
async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    // start a tokio tcp listener on OS port 0 to get a random port number
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port.");
    // extract the port number from the TCP socket server
    let port = listener.local_addr().unwrap().port();
    // generate the address for the application to run on
    let address = format!("http://127.0.0.1:{}", port);
    // get configuration from a configuration file
    let mut configuration = get_configuration().expect("Failed to read configuration");
    // randomize the database name so it will be different for each test run,
    // otherwise the insert operation won't succeed after the first test run,
    // due to the "UNIQUE" keyword in our database.
    configuration.database.database_name = Uuid::new_v4().to_string();
    // establish connection to our database
    let connection_pool = configure_database(&configuration.database).await;

    // get a server handler
    let server = zero2prod::startup::run(listener, connection_pool.clone())
        .expect("Failed to bind address.");
    // spawn the server asynchronously
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // connect to our database without a specific db name
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    // create a database using randomized database name
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    // connect to the database we just created
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    // perform database migrations
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    
    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    // spawn a server for testing
    let mut test_app = spawn_app().await;
    // initialize a mock client
    let client = reqwest::Client::new();
    // query a server address using GET method and get the reponse body
    let response = client
        .get(&format!("{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    // assertions of the test
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
    // Query our database for value stored
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")    
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
        
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    // paremetrised testing
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // since we are using parametrised testing,
        // we must provide a detailed description about any failures,
        // otherwise we won't be able to know which test case specifically causes the failure.
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}