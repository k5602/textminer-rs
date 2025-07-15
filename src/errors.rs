use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RedactrError {
    #[error("Model not loaded")]
    ModelNotLoaded,
    
    #[error("Text too long: {0} characters (max: {1})")]
    TextTooLong(usize, usize),
    
    #[error("Processing failed: {0}")]
    ProcessingFailed(String),
    
    #[error("Internal server error")]
    InternalError,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
    pub timestamp: String,
}

impl ResponseError for RedactrError {
    fn error_response(&self) -> HttpResponse {
        let status_code = match self {
            RedactrError::ModelNotLoaded => actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
            RedactrError::TextTooLong(_, _) => actix_web::http::StatusCode::BAD_REQUEST,
            RedactrError::ProcessingFailed(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            RedactrError::InternalError => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        let error_response = ErrorResponse {
            error: self.to_string(),
            code: status_code.as_u16(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        HttpResponse::build(status_code).json(error_response)
    }
}

impl From<anyhow::Error> for RedactrError {
    fn from(err: anyhow::Error) -> Self {
        RedactrError::ProcessingFailed(err.to_string())
    }
}
