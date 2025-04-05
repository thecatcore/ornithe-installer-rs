use std::fmt::Debug;

use log::error;
use tokio::task::JoinError;

#[derive(Debug)]
pub struct InstallerError(pub String);

impl From<eframe::Error> for InstallerError {
    fn from(value: eframe::Error) -> Self {
        InstallerError::handle_error(format!("{:?}", value))
    }
}

impl From<reqwest::Error> for InstallerError {
    fn from(value: reqwest::Error) -> Self {
        InstallerError::handle_error(format!("{:?}", value))
    }
}

impl From<serde_json::Error> for InstallerError {
    fn from(value: serde_json::Error) -> Self {
        InstallerError::handle_error(format!("{:?}", value))
    }
}

impl From<std::io::Error> for InstallerError {
    fn from(value: std::io::Error) -> Self {
        InstallerError::handle_error(format!("{:?}", value))
    }
}

impl InstallerError {
    fn handle_error(message: String) -> InstallerError {
        error!("{}", message);

        InstallerError(message)
    }
}
