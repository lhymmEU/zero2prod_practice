use std::net::TcpListener;

// this function handles the logic of spawn a server to the background
fn spawn_app() -> String {
    // start a tokio tcp listener on OS port 0 to get a random port number
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port.");
    // extract the port number from the TCP socket server
    let port = listener.local_addr().unwrap().port();
    // get a server handler from the library
    let server = zero2prod::run(listener).expect("Failed to bind address.");
    // spawn the server
    let _ = tokio::spawn(server);
    // return the actual address the server is running on
    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_works() {
    // spawn a server thread for testing,
    // returns a string represents the address the spawned server is running on
    let address = spawn_app();

    let client = reqwest::Client::new();
    // query a server address and get the reponse body
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // assertions of the test
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}