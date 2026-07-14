mod database;
mod service;
mod settings;

use axum::{Router, routing::{get, post}};

use crate::settings::Settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize service state
    let settings = Settings::new()?;
    let pool = database::create_pool(settings.database).await?;
    let state = service::State {
        settings: settings.api,
        pool: pool,
    };

    // Set up service endpoints
    let app = Router::new()
        .route("/documents", post(service::document::process))
        .route("/documents/{doc_id}/analytics", get(service::document::analytics))
        .route("/documents/{doc_id}/visits", get(service::document::visits))
        .route("/urls/{url_id}/visits", get(service::url::visits))
        .route("/visit/{url_id}", get(service::visit::process))
        .with_state(state)
        .into_make_service_with_connect_info::<std::net::SocketAddr>();

    // Start service
    let service_address = (settings.address.ip, settings.address.port);
    let listener = tokio::net::TcpListener::bind(service_address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
