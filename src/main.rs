mod errors;
mod handlers;
mod middleware;
mod models;
mod utils;

use actix_web::{
    middleware::Logger,
    web::{self, Data},
    App, HttpServer, HttpResponse, Responder,
};
use log::{info, error};
use rust_bert::pipelines::ner::NERModel;
use std::sync::Arc;
use std::io;
use tokio::sync::Mutex;
use crate::utils::init_logger;

/* ---------- Request / Response DTOs ---------- */

#[derive(Debug, serde::Deserialize)]
struct RedactRequest {
    text: String,
}

#[derive(Debug, serde::Serialize)]
struct RedactResponse {
    redacted_text: String,
    processing_time_ms: u128,
}

#[derive(Debug, serde::Serialize)]
struct HealthResponse {
    status: String,
    model_loaded: bool,
}

#[derive(Debug, serde::Serialize)]
struct ErrorResponse {
    error: String,
    code: u16,
    timestamp: String,
}

/* ---------- Model helper ---------- */

/// Loads the NER model asynchronously in a blocking task
async fn load_model() -> anyhow::Result<Arc<Mutex<NERModel>>> {
    let model = tokio::task::spawn_blocking(|| {
        let start = std::time::Instant::now();
        let model = NERModel::new(Default::default())?;
        info!("NER model loaded in {} ms", start.elapsed().as_millis());
        anyhow::Ok(model)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Failed to join model loading task: {}", e))??;
    
    Ok(Arc::new(Mutex::new(model)))
}

/* ---------- Entrypoint ---------- */

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    init_logger();

    info!("Booting Redactr...");

    // Load the BERT model
    let model = load_model().await.map_err(|e| {
        error!("Failed to load model: {}", e);
        std::io::Error::new(io::ErrorKind::Other, e.to_string())
    })?;

    let model_data = Data::new(model);

    info!("Starting server on 0.0.0.0:8080");
    
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(model_data.clone())
            // Register handlers from the handlers module
            .service(handlers::index)
            .service(handlers::health)
            .service(handlers::redact_text)
            .service(handlers::batch_redact_text)
            // Default 404 handler
            .default_service(
                web::route().to(|| async { 
                    HttpResponse::NotFound().json(crate::errors::ErrorResponse {
                        error: "Not Found".to_string(),
                        code: 404,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    })
                }),
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
