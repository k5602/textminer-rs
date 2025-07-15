use serde::{Deserialize, Serialize};

/// Options for redaction behavior
#[derive(Debug, Deserialize, Default, Clone)]
pub struct RedactionOptions {
    /// Whether to include detailed entity information in the response
    #[serde(default)]
    pub include_entity_details: bool,
    /// Whether to include confidence scores in the response
    #[serde(default = "default_true")]
    pub include_confidence: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Deserialize)]
pub struct RedactRequest {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct RedactResponse {
    pub redacted_text: String,
    pub processing_time_ms: u128,
    pub entities_found: usize,
    pub entity_types: Vec<String>,
    pub confidence_scores: Vec<f64>,
}

#[derive(Debug, Deserialize)]
pub struct BatchRedactRequest {
    /// List of texts to process
    pub texts: Vec<String>,
    /// Optional configuration for redaction
    #[serde(default)]
    pub options: RedactionOptions,
}

#[derive(Debug, Serialize)]
pub struct BatchRedactResponse {
    /// List of redaction results in the same order as input texts
    pub results: Vec<RedactResponse>,
    /// Total processing time in milliseconds
    pub total_processing_time_ms: u128,
    /// Total number of entities found across all texts
    pub total_entities_found: usize,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub model_loaded: bool,
    pub version: &'static str,
}

impl Default for HealthResponse {
    fn default() -> Self {
        Self {
            status: "ok".to_string(),
            model_loaded: true,
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}
