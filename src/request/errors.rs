use thiserror::Error;

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("invalid url: {0}")]
    InvalidUrl(String),
}
