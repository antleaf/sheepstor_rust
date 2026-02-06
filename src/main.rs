mod server;
mod cli;
mod logging;
mod website;
mod utilities;
mod git;
mod website_registry;
mod website_builders;
mod errors;
mod github_webhook;
mod ingest;

use crate::server::run_http_server;
use clap::Parser;
use crate::cli::{Cli, Commands};
use crate::logging::configure_flexi_logger;


#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    configure_flexi_logger(cli.global_opts.debug).expect("Failed to configure logger - quitting");
    log::info!("Starting process");
    let mut registry = website_registry::WebsiteRegistry::config(cli.global_opts.config).expect("Failed to load website registry configuration - quitting");
    registry.initialise().expect("Failed to initialise the website registry - quitting");
    log::info!("Loaded {} websites", registry.count());

    match &cli.commands {
        Commands::Server { port } => {
            log::info!("Running Server on port: {port}");
            run_http_server(*port, registry).await;
        }
        Commands::Update { sites } => {
            log::info!("Updating site(s): {sites}");
            match sites.as_str() {
                "all" => {
                    log::info!("Updating all sites");
                    match registry.process_all_websites() {
                        Ok(_) => log::info!("Website batch update completed"),
                        Err(e) => log::error!("Failed to update all websites: {e}"),
                    }
                }
                _ => {
                    let site_list: Vec<&str> = sites.split(',').collect();
                    for site_id in site_list {
                        let website = registry.get_website_by_id(site_id);
                        match website {
                            Some(website) => {
                                log::info!("Processing website: {}", website.id);
                                match registry.process_website(website) {
                                    Ok(_) => log::info!("Website '{}' updated successfully", website.id),
                                    Err(e) => log::error!("Failed to update website '{}': {}", website.id, e),
                                }
                            }
                            None => {
                                log::warn!("Website '{site_id}' not found in registry");
                                continue;
                            }
                        }
                    }
                }
            }
        }
        Commands::Push { site } => {
            log::info!("Pushing site: {site}");
            let website = registry.get_website_by_id(site);
            match website {
                Some(website) => {
                    log::info!("Processing website: {}", website.id);
                    match registry.push_website(website) {
                        Ok(_) => log::info!("Website changes '{}' pushed successfully", website.id),
                        Err(e) => log::error!("Failed to push changes to website '{}': {}", website.id, e),
                    }
                }
                None => {
                    log::warn!("Website '{site}' not found in registry");
                }
            }
        }
    }
    log::info!("Process Completed");
}
