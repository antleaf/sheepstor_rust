use crate::git::GitRepository;
use crate::website_builders::{build_index, build_with_hugo, build_with_verbatim_copy};
use std::fs;
use std::fs::create_dir_all;
use std::os::unix::fs::symlink;
use serde::{Deserialize};
use strum_macros::EnumString;

#[derive(Clone, EnumString, Debug, Deserialize)]
pub enum ContentProcessor {
    Unknown,
    Hugo,
    None,
    // Add other content processors as needed e.g.. Jekyll, MkDocs, etc.
}

#[derive(Clone, Deserialize)]
pub struct Website {
    pub id: String,
    pub content_processor: ContentProcessor,
    pub processor_root: String,
    pub github_webhook_secret_env_key: String,
    #[serde(default)]
    pub webroot: String,
    pub index: bool,
    pub git: GitRepository,
}

impl Website {

    pub fn build(&self) -> Result<(), Box<dyn std::error::Error>> {
        create_dir_all(std::path::Path::new(&self.webroot).join("logs"))?;
        let mut target_folder_for_build = std::path::Path::new(&self.webroot).join("public_1");
        let target_folder_symlink_path = std::path::Path::new(&self.webroot).join("public");
        match fs::read_link(&target_folder_symlink_path) {
            Ok(path) => {
                if path == target_folder_for_build {
                    target_folder_for_build = std::path::Path::new(&self.webroot).join("public_2");
                }
            }
            Err(_e) => {
                log::debug!("No symlink found at: {}, creating new one", &target_folder_symlink_path.display());
            }
        }
        if target_folder_for_build.exists() {
            fs::remove_dir_all(&target_folder_for_build)?;
        }
        create_dir_all(&target_folder_for_build)?;

        match self.content_processor {
            ContentProcessor::Hugo => {
                build_with_hugo(self, &target_folder_for_build)?;
                log::info!("Website: {} built using Hugo in folder: {}", self.id, &target_folder_for_build.display());
            }
            ContentProcessor::None => {
                log::debug!("Building website: {} without processor (using verbatim copy)", self.id);
                build_with_verbatim_copy(self, &target_folder_for_build)?;
            }
            ContentProcessor::Unknown => {
                log::error!("Unrecognised content processor for website: {}", self.id);
                return Err("Unrecognised content processor for website".into());
            }
        }
        if self.index {
            log::debug!("Building index for website: {}...", self.id);
            build_index(&target_folder_for_build)?;
        }
        if target_folder_symlink_path.exists() {
            fs::remove_file(&target_folder_symlink_path)?;
        }
        symlink(target_folder_for_build, target_folder_symlink_path)?;
        Ok(())
    }

    pub fn update_sources(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("Updating sources for website: {}", self.id);
        let git_repo_path = std::path::Path::new(&self.git.working_dir).join(".git");
        // Check if the git repository exists, if not, clone it
        if !git_repo_path.exists() {
            log::debug!("Cloning repository for website: {}", self.id);
            self.git.git_clone()?;
            log::debug!("Repository cloned for website: {}", self.id);
            return Ok(());
        }
        self.git.git_pull()?;
        log::debug!("Sources updated for website: {}", self.id);
        Ok(())
    }

    pub fn push(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("Pushing changes to website: {}", self.id);
        self.git.git_push()?;
        log::debug!("Changes pushed to website: {}", self.id);
        Ok(())
    }
}
