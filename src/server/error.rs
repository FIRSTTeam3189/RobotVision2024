use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError { 
    #[error("Server Listener Bind Failed")]
    BindFailed
}

#[derive(Error, Debug)]
pub enum ServerConfigError {
    #[error("Failed To Load Server Config")]
    FailedToLoadConfig
}