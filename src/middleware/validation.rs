use crate::errors::RedactrError;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;

const MAX_TEXT_LENGTH: usize = 10_000;

#[derive(Clone)]
pub struct TextLengthValidator;

impl<S, B> Transform<S, ServiceRequest> for TextLengthValidator
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = TextLengthValidatorMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TextLengthValidatorMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct TextLengthValidatorMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for TextLengthValidatorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            if let Some(body) = req.extract::<String>().await.ok() {
                if body.len() > MAX_TEXT_LENGTH {
                    return Err(RedactrError::TextTooLong(body.len(), MAX_TEXT_LENGTH).into());
                }
            }

            service.call(req).await
        })
    }
}

pub fn validate_text_length(text: &str) -> Result<(), RedactrError> {
    if text.len() > MAX_TEXT_LENGTH {
        return Err(RedactrError::TextTooLong(text.len(), MAX_TEXT_LENGTH));
    }
    Ok(())
}
