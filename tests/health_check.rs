use std::net::TcpListener;


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