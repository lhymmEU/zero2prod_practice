use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
    // spawn a server for testing
    let test_app = spawn_app().await;
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