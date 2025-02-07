pub mod auth_layers;
mod configuration;
mod db;
mod web_service;
use anyhow::Result;
use axum::{
    middleware,
    routing::{get, post},
    Extension, Router,
};
use tower_http::cors::CorsLayer;

pub async fn setup_service() -> Result<Router> {
    let config = configuration::AuthConfiguration::load()?;
    let db_pool = db::get_connection_pool(&config.db_filename).await?;

    db::perform_migrations(db_pool.clone()).await?;

    let secure_router = Router::new()
        .layer(CorsLayer::very_permissive())
        .route("/users", get(web_service::list_users))
        .route("/users/:id", get(web_service::get_user))
        .route("/users/delete/:id", get(web_service::delete_user))
        .route("/users/add", post(web_service::add_user))
        .route("/users/update/:id", post(web_service::update_user))
        .layer(Extension(config.clone()))
        .layer(Extension(db_pool.clone()))
        .route_layer(middleware::from_fn(auth_layers::require_token));

    let router = Router::new()
        .layer(CorsLayer::very_permissive())
        .route("/login", post(web_service::do_login))
        .route("/is_token_valid/:token", get(web_service::is_token_valid))
        .nest("/", secure_router)
        .layer(Extension(config))
        .layer(Extension(db_pool));

    Ok(router)
}
