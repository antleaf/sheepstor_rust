use secrecy::SecretString;
use std::env;
use std::env::VarError;
use std::path::PathBuf;
use zip_extensions::zip_extract::zip_extract;
use chrono::offset::Utc;
use chrono::DateTime;
use std::time::SystemTime;

pub fn get_secret_from_env(key: String) -> Result<SecretString, VarError> {
    match env::var(&key) {
        Ok(value) => Ok(SecretString::from(value)),
        Err(e) => {
            log::error!("Error reading ENV variable with key: {key} - {e}");
            Err(e)
        }
    }
}

pub fn unpack_zipped_folder(zip_path: &str, destination: &str) -> Result<(), Box<dyn std::error::Error>> {
    let archive_file:PathBuf = zip_path.into();
    let destination_folder:PathBuf = destination.into();
    zip_extract(&archive_file, &destination_folder)?;
    Ok(())
}

pub fn interpolate_content_folder_path(content_root: String) -> String {
    let datetime: DateTime<Utc> = SystemTime::now().into();
    let interpolated_string = content_root.replace("{YEAR}", datetime.format("%Y").to_string().as_str());
    interpolated_string
}