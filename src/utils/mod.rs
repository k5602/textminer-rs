use crate::errors::RedactrError;
use rust_bert::pipelines::ner::NERModel;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn load_model() -> Result<Arc<Mutex<NERModel>>, RedactrError> {
    let model = tokio::task::spawn_blocking(|| {
        let start = std::time::Instant::now();
        let model = NERModel::new(Default::default())?;
        log::info!("NER model loaded in {} ms", start.elapsed().as_millis());
        anyhow::Ok(model)
    })
    .await
    .map_err(|e| RedactrError::ProcessingFailed(e.to_string()))??;

    Ok(Arc::new(Mutex::new(model)))
}

pub fn init_logger() {
    if std::env::var("RUST_LOG").is_err() {
        // Safe because we're setting a static string in a controlled environment
        unsafe {
            std::env::set_var("RUST_LOG", "info,actix_web=info");
        }
    }
    env_logger::init();
}
