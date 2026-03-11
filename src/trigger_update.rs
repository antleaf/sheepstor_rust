use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use secrecy::ExposeSecret;
use crate::github_webhook::ApplicationState;
use crate::utilities::get_secret_from_env;

pub async fn trigger_update(State(state): State<ApplicationState>, headers: HeaderMap, Path(website_id): Path<String>) -> Response {
    match state.registry.get_website_by_id(&website_id) {
        Some(website) => {
            log::info!("Processing website: {}", website.id);
            let token: String;
            let token_env_key = website.ingest_token_env_key.clone();
            match token_env_key {
                Some(env_key) => match get_secret_from_env(env_key) {
                    Ok(secret) => {
                        token = secret.expose_secret().to_string();
                    }
                    Err(_) => {
                        log::error!("Failed to retrieve ingest token from environment variable");
                        return (StatusCode::UNAUTHORIZED, "Failed to retrieve ingest token").into_response();
                    }
                },
                None => {
                    log::error!("Failed to retrieve ingest token environment variable from Sheepstor config");
                    return (StatusCode::UNAUTHORIZED, "Failed to retrieve ingest token").into_response();
                }
            }
            match headers.get("Authorization") {
                Some(auth_header) => {
                    let auth_str = auth_header.to_str().unwrap_or("");
                    if let Some(bearer_token) = auth_str.strip_prefix("Bearer ") {
                        let bearer_token = bearer_token.to_string();
                        if token == bearer_token {
                            log::debug!("Successfully verified ingest token");
                        } else {
                            log::error!("Invalid ingest token provided");
                            return (StatusCode::UNAUTHORIZED, "Invalid ingest token").into_response();
                        }
                    } else {
                        log::error!("Invalid Authorization header format");
                        return (StatusCode::UNAUTHORIZED, "Invalid Authorization header").into_response();
                    }
                }
                None => {
                    log::error!("Missing Authorization header");
                    return (StatusCode::UNAUTHORIZED, "Missing Authorization header").into_response();
                }
            }
            match state.registry.process_website(website) {
                Ok(_) => log::info!("Website '{}' updated successfully", website.id),
                Err(e) => log::error!("Failed to update website '{}': {}", website.id, e),
            }
        }
        None => {
            log::warn!("Website with id: {website_id} not found in registry");
            return (StatusCode::NOT_FOUND, format!("Website {} not found", &website_id)).into_response();
        }
    }
    (StatusCode::CREATED, "OK").into_response()
}