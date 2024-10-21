use std::{env, path::PathBuf};

use crate::errors::Error;
use rfd::AsyncFileDialog;
pub const DEFAULT_LOCATION: &str = ".config/solana/id.json";

pub fn default_file() -> PathBuf {
    let home_dir = env::var("HOME") // mac users
        .or_else(|_| env::var("USERPROFILE")) // windows users
        .expect("Cannot find home directory");
    let mut path = PathBuf::from(home_dir);
    path.push(DEFAULT_LOCATION);
    path
}

pub async fn pick_file() -> Result<PathBuf, Error> {
    let handle = AsyncFileDialog::new()
        .set_title("Choose a valid json solana keypair")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;

    if handle.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
        return Err(Error::InvalidFileType);
    }
    Ok(handle.path().to_owned())
}

