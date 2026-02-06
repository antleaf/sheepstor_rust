use std::error::Error;
use std::io::Write;
use axum::body::{Bytes};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use secrecy::ExposeSecret;
use crate::github_webhook::ApplicationState;
use crate::utilities::{get_secret_from_env, interpolate_content_folder_path, unpack_zipped_folder};

pub async fn ingest_zip_file(State(state): State<ApplicationState>, headers: HeaderMap, Path(website_id): Path<String>, body: Bytes) -> Response {
    match state.registry.get_website_by_id(&website_id) {
        Some(website) => {
            log::info!("Processing website: {}", website.id);
            let token: String ;
            let token_env_key = website.ingest_token_env_key.clone();
            match token_env_key {
                Some(env_key) => {
                    match get_secret_from_env(env_key) {
                        Ok(secret) => {
                            token = secret.expose_secret().to_string();
                        }
                        Err(_) => {
                            log::error!("Failed to retrieve ingest token from environment variable");
                            return (StatusCode::UNAUTHORIZED, "Failed to retrieve ingest token").into_response();
                        }
                    }
                }
                None => {
                    log::error!("Failed to retrieve ingest token environment variable from Sheepstor config");
                    return (StatusCode::UNAUTHORIZED, "Failed to retrieve ingest token").into_response();
                }
            }
            
            match headers.get("Authorization") {
                Some(auth_header) => {
                    let auth_str = auth_header.to_str().unwrap_or("");
                    if auth_str.starts_with("Bearer ") {
                        let bearer_token = auth_str[7..].to_string();
                        if token == bearer_token {
                            log::debug!("Successfully verified ingest token");
                            let upload_zip_file_path = state.registry.tmp_folder.clone() + "/upload.zip";
                            log::debug!("Creating new temp file for uploaded zip at path: {}", upload_zip_file_path);
                            match std::fs::File::create(&upload_zip_file_path) {
                                Ok(mut file) => {
                                    match (file.write_all(&body)) {
                                        Ok(_) => {
                                            let target_content_folder = website.git.working_dir.clone() + "/" + &interpolate_content_folder_path(website.content_root.clone());
                                            match unpack_zipped_folder(&upload_zip_file_path, &target_content_folder) {
                                                Ok(_) => {
                                                    log::debug!("Unpacking zip file: {} into folder: {}", upload_zip_file_path,target_content_folder);
                                                }
                                                Err(_) => {return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save uploaded zip file").into_response();}
                                            }
                                        },
                                        Err(e) => {
                                            log::error!("Failed to write data to zip file: {}", e);
                                            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save uploaded zip file").into_response();
                                        }
                                    }
                                    match std::io::copy(&mut body.as_ref(), &mut file) {
                                        Ok(_) => {
                                            log::debug!("Successfully saved uploaded zip file to {}", upload_zip_file_path);
                                            
                                        }
                                        Err(e) => {
                                            log::error!("Failed to save uploaded zip file: {}", e);
                                            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save uploaded zip file").into_response();
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to create tmp file for uploaded zip: {}", e);
                                    return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to process uploaded file").into_response();
                                }
                            }
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
        }
        None => {
            log::warn!("Website with id: {website_id} not found in registry");
            return (StatusCode::NOT_FOUND, format!("Website {} not found", &website_id)).into_response();
        }
    }
    (StatusCode::CREATED, "OK").into_response()
}