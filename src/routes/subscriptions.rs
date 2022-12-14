use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

/// Store a new subscriber to our database.
/// Notice that, the two arguments are extractors provided by the actix_web framework,
/// they are automatically populated when a request comes in,
/// don't need to provide the values manually (see startup.rs for an example)
/// 
/// # Arguments
/// 
/// * `form` - Extracted values from a url encoded request payload.
/// * `pool` - Extracted values from a shared database connection pool.
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>
) -> HttpResponse {
    // Insert the new subscriber into our database
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
