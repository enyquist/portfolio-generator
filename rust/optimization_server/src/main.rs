// src/main.rs

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting optimization server at http://127.0.0.1:8080");
    optimization_server::run_server().await
}
