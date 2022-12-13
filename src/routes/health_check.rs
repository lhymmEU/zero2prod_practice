use actix_web::HttpResponse;

// a dummy function to test if the bare minimum of code works
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
