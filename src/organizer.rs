use log::{error, info};
use serde::{Deserialize, Serialize};

#[cfg(target_family = "unix")]
use std::os::unix::fs::symlink;
#[cfg(target_family = "windows")]
use std::os::windows::fs::symlink_file;

use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum OrganizationMode {
    Symlinks,
    Move,
    Copy,
}

pub trait OrganizationStrategy {
    fn organize(&self, _original_file: &PathBuf, _destination: &PathBuf) {}
}

pub struct CopyStrategy {}
impl CopyStrategy {
    pub fn new() -> CopyStrategy {
        CopyStrategy {}
    }
}
impl OrganizationStrategy for CopyStrategy {
    fn organize(&self, original_file: &PathBuf, destination: &PathBuf) {
        info!(
            "Copying {} to {}",
            original_file.display(),
            destination.display()
        );

        if let Some(parent) = destination.parent() {
            let _ = std::fs::create_dir_all(parent);
        };

        if destination.exists() {
            error!("Destination file already exists. Aborting copy.");
            return;
        }

        let copy_result = std::fs::copy(original_file, destination);

        match copy_result {
            Ok(_) => info!("File copied successfully"),
            Err(e) => error!("Error copying file: {}", e),
        }
    }
}
pub struct SymlinksStrategy {}
impl SymlinksStrategy {
    pub fn new() -> SymlinksStrategy {
        SymlinksStrategy {}
    }
}

#[cfg(target_family = "unix")]
fn make_symlink(original_file: &PathBuf, destination: &PathBuf) {
    let symlink_result = symlink(original_file, destination);
    match symlink_result {
        Ok(_) => info!("File symlinked successfully"),
        Err(e) => error!("Error symlinking file: {}", e),
    }
}
#[cfg(target_family = "windows")]
fn make_symlink(original_file: &PathBuf, destination: &PathBuf) {
    let symlink_result = symlink_file(original_file, destination);
    match symlink_result {
        Ok(_) => info!("File symlinked successfully"),
        Err(e) => error!("Error symlinking file: {}", e),
    }
}
impl OrganizationStrategy for SymlinksStrategy {
    fn organize(&self, original_file: &PathBuf, destination: &PathBuf) {
        info!(
            "Symlinking {} to {}",
            destination.display(),
            original_file.display(),
        );
        if let Some(parent) = destination.parent() {
            let _ = std::fs::create_dir_all(parent);
        };

        if destination.exists() {
            error!("Destination file already exists. Aborting copy.");
            return;
        }

        make_symlink(original_file, destination);
    }
}
pub struct MoveStrategy {}

impl MoveStrategy {
    pub fn new() -> MoveStrategy {
        MoveStrategy {}
    }
}
impl OrganizationStrategy for MoveStrategy {
    fn organize(&self, original_file: &PathBuf, destination: &PathBuf) {
        info!(
            "Moving {} to {}",
            original_file.display(),
            destination.display(),
        );
        if let Some(parent) = destination.parent() {
            let _ = std::fs::create_dir_all(parent);
        };

        if destination.exists() {
            error!("Destination file already exists. Aborting copy.");
            return;
        }
        let move_result = std::fs::rename(original_file, destination);
        match move_result {
            Ok(_) => info!("File moved successfully"),
            Err(e) => error!("Error moving file: {}", e),
        }
    }
}
