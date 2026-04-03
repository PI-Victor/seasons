use crate::hue::models::HueApiErrorBody;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HueError {
    #[error("invalid Hue bridge configuration: {0}")]
    InvalidConfig(&'static str),
    #[error("Hue bridge username is required for this operation")]
    MissingUsername,
    #[error("Hue bridge returned an unexpected response: {0}")]
    UnexpectedResponse(&'static str),
    #[error("failed to communicate with the Hue bridge: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Hue bridge API error {code} at {address}: {description}")]
    Api {
        code: u16,
        address: String,
        description: String,
    },
}

impl From<HueApiErrorBody> for HueError {
    fn from(value: HueApiErrorBody) -> Self {
        Self::Api {
            code: value.error_type,
            address: value.address,
            description: value.description,
        }
    }
}
