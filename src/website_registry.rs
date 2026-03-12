use crate::website::Website;
use serde::{Deserialize};
use std::fs;

#[derive(Clone, Deserialize)]
pub struct WebsiteRegistry {
    pub source_root: String,
    pub docs_root: String,
    // pub tmp_folder: String,
    pub websites: Vec<Website>,
}

impl WebsiteRegistry {
    pub fn config(config_file_path: String) -> Result<WebsiteRegistry, Box<dyn std::error::Error>> {
        let path = std::path::Path::new(&config_file_path);
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(err) => {
                log::error!("Couldn't open {}: {}", path.display(), err);
                return Err(err.into());
            }
        };
        let registry: WebsiteRegistry = match serde_yaml::from_reader(file) {
            Ok(registry) => {
                log::info!("Loaded config from file at {}", path.display());
                registry
            }
            Err(err) => {
                log::error!("Error deserializing YAML from config file at {}: {}", path.display(), err);
                return Err(err.into());
            }
        };
        Ok(registry)
    }

    pub fn initialise(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.source_root)?;
        fs::create_dir_all(&self.docs_root)?;
        for website in &mut self.websites {
            website.processor_root = std::path::Path::new(&self.source_root).join(&website.id).join(&website.processor_root).display().to_string();
            website.webroot = std::path::Path::new(&self.docs_root).join(&website.id).display().to_string();
            website.git.working_dir = std::path::Path::new(&self.source_root).join(&website.id).display().to_string();
        }
        Ok(())
    }

    pub fn count(&self) -> u8 {
        self.websites.len() as u8
    }

    pub fn get_website_by_id(&self, id: &str) -> Option<&Website> {
        self.websites.iter().find(|w| w.id == id)
    }

    pub fn push_website(&self, website: &Website) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("Pushing website: {}...", website.id);
        website.push()?;
        Ok(())
    }

    pub fn process_website(&self, website: &Website) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("Processing website: {}...", website.id);
        website.update_sources()?;
        website.build()?;
        Ok(())
    }

    pub fn process_all_websites(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.websites.iter().for_each(|website| match self.process_website(website) {
            Ok(_) => log::info!("Website {} processed successfully", website.id),
            Err(e) => log::error!("Failed to process website '{}': {}", website.id, e),
        });
        Ok(())
    }
}
