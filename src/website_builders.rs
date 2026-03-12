use crate::website::Website;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub fn build_with_hugo(website: &Website, target_folder_for_build: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!(
        "Building website: {} using Hugo from processor root: {} into folder: {}",
        website.id,
        &website.processor_root,
        &target_folder_for_build.display()
    );
    let hugo_build_command_output = std::process::Command::new("hugo")
        .arg("--quiet")
        .arg("--ignoreCache")
        .arg("--source")
        .arg(&website.processor_root)
        .arg("--destination")
        .arg(target_folder_for_build)
        .output()?;
    if hugo_build_command_output.status.success() {
        Ok(())
    } else {
        let error_message = String::from_utf8_lossy(&hugo_build_command_output.stderr);
        log::error!("Hugo build failed for website {}: {}", website.id, error_message);
        Err(Box::new(std::io::Error::other(error_message)))
    }
}

pub fn build_with_verbatim_copy(website: &Website, target_folder_for_build: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let source_path = PathBuf::from(&website.processor_root);
    copy_dir_all(source_path, target_folder_for_build)?;
    Ok(())
}

pub fn build_index(target_folder_for_build: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let index_build_command_output = std::process::Command::new("pagefind").arg("--site").arg(target_folder_for_build).output()?;
    if index_build_command_output.status.success() {
        Ok(())
    } else {
        let error_message = String::from_utf8_lossy(&index_build_command_output.stderr);
        Err(Box::new(std::io::Error::other(error_message)))
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
