use actix_web::{
    App, HttpResponse, HttpServer, Responder, Result, get, post,
    web::{self, Json},
};
use log::{error, info};
use rust_bert::pipelines::ner::NERModel;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

/* ---------- Request / Response DTOs ---------- */

#[derive(Debug, Deserialize)]
struct RedactRequest {
    text: String,
}

#[derive(Debug, Serialize)]
struct RedactResponse {
    redacted_text: String,
    processing_time_ms: u128,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    model_loaded: bool,
}

/* ---------- Model helper ---------- */

/// Loads the NER model asynchronously in a blocking task
async fn load_model() -> anyhow::Result<Arc<Mutex<NERModel>>> {
    let model = tokio::task::spawn_blocking(|| {
        let start = Instant::now();
        let model = NERModel::new(Default::default())?;
        info!("NER model loaded in {} ms", start.elapsed().as_millis());
        anyhow::Ok(model)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Failed to join model loading task: {}", e))??;
    
    Ok(Arc::new(Mutex::new(model)))
}

/* ---------- Handlers ---------- */

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html"))
}

#[get("/api/health")]
async fn health(model: web::Data<Arc<Mutex<NERModel>>>) -> Result<Json<HealthResponse>> {
    let _model = model.lock().await;
    Ok(Json(HealthResponse {
        status: "ok".into(),
        model_loaded: true,
    }))
}

#[post("/api/redact")]
async fn redact_text(
    payload: Json<RedactRequest>,
    model: web::Data<Arc<Mutex<NERModel>>>,
) -> Result<Json<RedactResponse>> {
    let text = payload.text.trim();
    if text.is_empty() {
        return Ok(Json(RedactResponse {
            redacted_text: String::new(),
            processing_time_ms: 0,
        }));
    }

    let start = Instant::now();

    let model = model.clone();
    let text_clone = text.to_owned();

    let entities = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let model = model.lock().await;
            model.predict_full_entities(&[&text_clone])
        })
    })
    .await
    .map_err(|e| {
        error!("Task join error: {e}");
        actix_web::error::ErrorInternalServerError("processing failed")
    })?;

    let mut redacted = text.to_string();
    let mut all_entities: Vec<_> = entities.into_iter().flatten().collect();
    all_entities.sort_by_key(|e| std::cmp::Reverse(e.offset.begin));

    for entity in all_entities {
        let begin = entity.offset.begin as usize;
        let end = entity.offset.end as usize;

        if begin <= end && end <= redacted.len() {
            redacted.replace_range(begin..end, &format!("[{}]", entity.label));
        }
    }

    Ok(Json(RedactResponse {
        redacted_text: redacted.to_string(),
        processing_time_ms: start.elapsed().as_millis(),
    }))
}

/* ---------- Entrypoint ---------- */

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "info,actix_web=info");
        }
    }
    env_logger::init();

    info!("Booting Redactr…");

    let model = load_model().await.map_err(|e| {
        error!("Model init failed: {e}");
        std::io::Error::new(std::io::ErrorKind::Other, e)
    })?;

    let data = web::Data::new(model);

    info!("Starting server on 0.0.0.0:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(index)
            .service(health)
            .service(redact_text)
            .default_service(
                web::route().to(|| async { HttpResponse::NotFound().body("404 Not Found") }),
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
