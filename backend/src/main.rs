mod handlers;
mod stego;

use actix_web::{App, HttpServer};
use actix_cors::Cors;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Running on http://localhost:8080");
    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()  // For dev only! In production, replace with `.allowed_origin("https://your-frontend-domain")`
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .configure(handlers::init_routes)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

