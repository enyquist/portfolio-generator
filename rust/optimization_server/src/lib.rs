// src/lib.rs

pub mod handlers;
pub mod models;
pub mod objective;
pub mod utils;
pub mod taxbrackets;

pub async fn run_server() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};
    use handlers::{health_check, optimize};

    HttpServer::new(|| {
        App::new()
            .service(optimize)
            .service(health_check)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}