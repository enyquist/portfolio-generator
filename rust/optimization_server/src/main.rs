// src/main.rs

use actix_web::{App, HttpServer};
use optimization_server::handlers::{health_check, optimize};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting optimization server at http://127.0.0.1:8080");
    HttpServer::new(|| {
        App::new()
            .service(optimize)
            .service(health_check)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
