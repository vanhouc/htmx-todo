use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Router,
};
use axum_embed::ServeEmbed;
use axum_htmx::AutoVaryLayer;
use rust_embed::RustEmbed;
use sqlx::PgPool;
use thiserror::Error;
use tower_livereload::LiveReloadLayer;

mod todo;

#[tokio::main]
async fn main() {
    // setup tracing with the RUST_LOG environment variable
    tracing_subscriber::fmt::init();

    // create database connection pool
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL must exist"))
        .await
        .expect("database must exist and be reachable");

    // apply migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("database migrations must be applicable");

    // build application routes and middleware
    let mut router = Router::new()
        .nest("/todos", todo::router())
        .nest_service("/assets", ServeEmbed::<Assets>::new())
        .fallback(|| async { Redirect::permanent("/todos") })
        .layer(AutoVaryLayer);

    // If in debug mode add live reloading
    if cfg!(debug) {
        router = router.layer(LiveReloadLayer::new())
    }

    // Add state to our router
    let app = router.with_state(pool);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

type SharedState = PgPool;

#[derive(RustEmbed, Clone)]
#[folder = "assets/"]
struct Assets;

#[derive(Error, Debug)]
enum Error {
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    #[error(transparent)]
    Askama(#[from] askama::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "oops something went wrong",
        )
            .into_response()
    }
}
