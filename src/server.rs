use crate::website_registry::WebsiteRegistry;
use axum::{
    routing::{get, post},
    Router,
};
use crate::github_webhook::{process_github_webhook, ApplicationState};
use crate::ingest::ingest_zip_file;
use crate::trigger_update::trigger_update;

pub fn create_router(registry: WebsiteRegistry) -> Router {
    let state = ApplicationState { registry };
    Router::new()
        .route("/", get(|| async { "Sheepstor" }))
        .route("/health", get(|| async { "OK" }))
        .route("/update/{website_id}", post(process_github_webhook))
        .route("/trigger_update/{website_id}", get(trigger_update))
        .route("/ingest/{website_id}", post(ingest_zip_file))
        .with_state(state)
}

pub async fn run_http_server(port: u16, registry: WebsiteRegistry) {
    let router = create_router(registry);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.expect("Failed to bind to port");
    axum::serve(listener, router).await.expect("Failed to start http server");
}
