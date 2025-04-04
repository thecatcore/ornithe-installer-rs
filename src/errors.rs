use log::error;

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

impl InstallerError {
    fn handle_error(message: String) -> InstallerError {
        error!("{}", message);

        InstallerError(message)
    }
}
