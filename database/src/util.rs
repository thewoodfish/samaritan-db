use std::fs::{File, OpenOptions};
use std::io::Error;

/// Open a file for writing (append-mode). 
/// Create new if it doen't exist already
pub fn create_or_append_file(file_path: &str) -> Result<File, Error> {
    Ok(OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?)
}
