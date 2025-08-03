mod handlers;
mod stego;
use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Running on http://localhost:8080");
    HttpServer::new(|| {
        App::new()
            .configure(handlers::init_routes)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
