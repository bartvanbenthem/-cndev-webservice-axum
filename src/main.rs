mod auth;
mod content;
mod mail;
mod service_config;
use anyhow::Result;
use axum::Extension;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let service_settings = service_config::ServiceConfig::load()?;
    let auth_router = auth::setup_service().await?;
    let content_router = content::setup_service(service_settings.listen_port.clone()).await?;
    let mail_router = mail::setup_service().await?;

    // Listen address from configuration
    let listen_address = format!(
        "{}:{}",
        service_settings.listen_address, service_settings.listen_port
    );
    let listener = tokio::net::TcpListener::bind(&listen_address).await?;
    tracing::info!("Listening on {}", listen_address);

    // The default web server
    let static_content = ServiceBuilder::new()
        .layer(CorsLayer::very_permissive())
        .service(ServeDir::new(&service_settings.static_content));

    // Build the master router
    let master_router = axum::Router::new()
        .layer(CorsLayer::very_permissive())
        .nest("/api/v1/auth", auth_router)
        .nest("/api/v1/content", content_router)
        .nest("/api/v1/mail", mail_router)
        .layer(Extension(service_settings))
        .nest_service("/", static_content);

    // Launch Axum
    axum::serve(listener, master_router).await?;

    Ok(())
}
