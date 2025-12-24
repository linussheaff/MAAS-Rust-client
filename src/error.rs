use thiserror::Error;

#[derive(Error, Debug)]
pub enum MaasError {
    // When API key is not in the correct format
    #[error("Invalid API key, expected 3 parts seperated in form A:B:C")]
    InvalidKeyFormat,

    // When server rejects credentials (401)
    #[error("Authentication failed, check API key")]
    Unauthorized,

    // Network level errors
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    // When server returns an error
    #[error("MAAS API error {status}: {body}")]
    ApiError{
        status: reqwest::StatusCode,
        body: String,
    },

    // JSON can't be parsed into the expected struct
    #[error("Failed to parse JSON: {0}")]
    Serialization(#[from] serde_json::Error),

    // URL parsing has failed
    #[error("Invalid URL: {0}")]
    UrlParseError(#[from] url::ParseError),
}