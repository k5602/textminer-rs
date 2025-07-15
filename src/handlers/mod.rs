use crate::{
    errors::RedactrError,
    models::{BatchRedactRequest, BatchRedactResponse, HealthResponse, RedactRequest, RedactResponse, RedactionOptions},
};
use actix_web::{get, post, web, HttpResponse, Responder, Result};
use rust_bert::pipelines::ner::NERModel;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

#[get("/api/health")]
pub async fn health(model: web::Data<Arc<Mutex<NERModel>>>) -> impl Responder {
    let _model = model.lock().await;
    HttpResponse::Ok().json(HealthResponse::default())
}

#[post("/api/redact")]
pub async fn redact_text(
    payload: web::Json<RedactRequest>,
    model: web::Data<Arc<Mutex<NERModel>>>,
) -> Result<impl Responder, RedactrError> {
    let text = payload.text.trim();
    if text.is_empty() {
        return Ok(HttpResponse::Ok().json(RedactResponse {
            redacted_text: String::new(),
            processing_time_ms: 0,
            entities_found: 0,
            entity_types: Vec::new(),
            confidence_scores: Vec::new(),
        }));
    }

    let start = Instant::now();
    
    let model_clone = model.clone();
    let text_clone = text.to_string();

    let entities = match tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let model = model_clone.lock().await;
            model.predict_full_entities(&[&text_clone])
        })
    })
    .await
    .map_err(|e| RedactrError::ProcessingFailed(e.to_string())) {
        Ok(entities) => entities,
        Err(e) => return Err(e)
    };
    
    let mut redacted = text.to_string();
    let mut all_entities: Vec<_> = entities.into_iter().flatten().collect();
    all_entities.sort_by_key(|e| std::cmp::Reverse(e.offset.begin));
    
    // Process entities in reverse order to maintain correct positions when replacing
    for entity in &all_entities {
        let begin = entity.offset.begin as usize;
        let end = entity.offset.end as usize;
        
        if begin <= end && end <= redacted.len() {
            redacted.replace_range(begin..end, &format!("[{}]", entity.label));
        }
    }
    
    // Collect entity types and confidence scores
    let entity_types: Vec<_> = all_entities.iter().map(|e| e.label.clone()).collect();
    let confidence_scores: Vec<_> = all_entities.iter().map(|e| e.score).collect();

    Ok(HttpResponse::Ok().json(RedactResponse {
        redacted_text: redacted,
        processing_time_ms: start.elapsed().as_millis(),
        entities_found: entity_types.len(),
        entity_types,
        confidence_scores,
    }))
}

/// Process a batch of texts for PII redaction
/// 
/// This endpoint accepts multiple texts in a single request and processes them efficiently
/// using the NER model. It's optimized for throughput rather than low latency.
#[post("/api/redact/batch")]
pub async fn batch_redact_text(
    payload: web::Json<BatchRedactRequest>,
    model: web::Data<Arc<Mutex<NERModel>>>,
) -> Result<impl Responder, RedactrError> {
    let start = Instant::now();
    let options = &payload.options;
    
    // Early return for empty batch
    if payload.texts.is_empty() {
        return Ok(HttpResponse::Ok().json(BatchRedactResponse {
            results: Vec::new(),
            total_processing_time_ms: 0,
            total_entities_found: 0,
        }));
    }

    // Process texts in a blocking task to avoid blocking the async runtime
    let model_clone = model.clone();
    let texts_clone = payload.texts.clone();
    let options_clone = options.clone();
    
    // Process the batch in a blocking task
    let process_result = tokio::task::spawn_blocking(move || {
        // Create a new runtime for the blocking task
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            RedactrError::ProcessingFailed(format!("Failed to create runtime: {}", e))
        })?;
        
        // Process the batch
        let results = rt.block_on(async {
            let model = model_clone.lock().await;
            
            // Process all texts in a single batch for better performance
            let predictions = model.predict_full_entities(
                &texts_clone.iter().map(String::as_str).collect::<Vec<_>>()
            );
            
            // Process each text with its predictions
            texts_clone.into_iter()
                .zip(predictions.into_iter())
                .map(|(text, entities)| {
                    let mut redacted = text.clone();
                    let mut all_entities = entities;
                    all_entities.sort_by_key(|e| std::cmp::Reverse(e.offset.begin));
                    
                    // Apply redactions in reverse order to maintain correct positions
                    for entity in &all_entities {
                        let begin = entity.offset.begin as usize;
                        let end = entity.offset.end as usize;
                        
                        if begin <= end && end <= redacted.len() {
                            redacted.replace_range(begin..end, &format!("[{}]", entity.label));
                        }
                    }
                    
                    // Prepare response based on options
                    let entity_types: Vec<_> = all_entities.iter()
                        .map(|e| e.label.clone())
                        .collect();
                        
                    let confidence_scores = if options_clone.include_confidence {
                        all_entities.iter()
                            .map(|e| e.score)
                            .collect()
                    } else {
                        Vec::new()
                    };
                    
                    RedactResponse {
                        redacted_text: redacted,
                        processing_time_ms: 0, // Will be set by the batch timing
                        entities_found: entity_types.len(),
                        entity_types,
                        confidence_scores,
                    }
                })
                .collect::<Vec<_>>()
        });
        
        Ok::<_, RedactrError>(results)
    }).await;
    
    // Handle the result of the blocking task
    let results = match process_result {
        Ok(Ok(results)) => results,
        Ok(Err(e)) => return Err(e),
        Err(e) => return Err(RedactrError::ProcessingFailed(e.to_string())),
    };
    
    let total_processing_time = start.elapsed();
    let total_entities = results.iter().map(|r| r.entities_found).sum();
    
    Ok(HttpResponse::Ok().json(BatchRedactResponse {
        results,
        total_processing_time_ms: total_processing_time.as_millis(),
        total_entities_found: total_entities,
    }))
}

#[get("/")]
pub async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../../static/index.html"))
}
