use serde::{Deserialize};

#[derive(Clone,Deserialize)]
pub struct GitRepository {
    pub clone_id: String,
    pub branch: String,
    #[serde(default)]
    pub working_dir: String,
}

impl GitRepository {

    pub fn git_pull(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("Pulling latest changes for repository {} at branch {}", self.clone_id, self.branch);
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(&self.working_dir)
            .arg("pull")
            .output()?;

        if output.status.success() {
            log::debug!("Repository pulled successfully");
            Ok(())
        } else {
            let error_message = String::from_utf8_lossy(&output.stderr);
            Err(Box::new(std::io::Error::other(error_message)))
        }
    }

    pub fn git_clone(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("Cloning repository {} at branch {} into {}", self.clone_id, self.branch, self.working_dir);
        let output = std::process::Command::new("git")
            .arg("clone")
            .arg("-b")
            .arg(&self.branch)
            .arg(&self.clone_id)
            .arg(&self.working_dir)
            .output()?;

        if output.status.success() {
            log::debug!("Repository cloned successfully");
            Ok(())
        } else {
            let error_message = String::from_utf8_lossy(&output.stderr);
            Err(Box::new(std::io::Error::other(error_message)))
        }
    }
}