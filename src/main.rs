use actix_web::{middleware, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use backend::Pool;
use std::sync::Arc;

use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let port = std::env::var("PORT").expect("PORT must be set");
    let http_url = ["0.0.0.0", port.as_str()].join(":");

    env_logger::init();

    HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(backend::handlers::validator);
        App::new()
            .wrap(middleware::Logger::default())
            .data(pool.clone())
            .route("/login", web::post().to(backend::handlers::login))
            .route(
                "/register",
                web::post().to(backend::handlers::register_user),
            )
            //playground without authentication
            .route(
                "/graphql",
                web::get().to(backend::handlers::graphql_playground),
            )
            .service(
                web::resource("/graphql")
                    .wrap(auth)
                    .route(web::post().to(backend::handlers::graphql))
                    .data(Arc::new(backend::graphql::Schema::new(
                        backend::graphql::QueryRoot,
                        backend::graphql::MutationRoot,
                    ))),
            )
    })
    .bind(http_url.as_str())?
    .run()
    .await
}
