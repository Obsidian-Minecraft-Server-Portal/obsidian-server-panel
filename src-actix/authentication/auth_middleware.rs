use crate::authentication::auth_data::UserData;
use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, error::ErrorUnauthorized, Error, HttpMessage};
use futures::future::LocalBoxFuture;
use std::rc::Rc;

pub struct Authentication;

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthenticationMiddleware<S>;
    type InitError = ();
    type Future = std::pin::Pin<Box<dyn Future<Output = Result<Self::Transform, Self::InitError>> + 'static>>;

    fn new_transform(&self, service: S) -> Self::Future {
        Box::pin(async move { Ok(AuthenticationMiddleware { service: Rc::new(service) }) })
    }
}
pub struct AuthenticationMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            let headers = req.headers().clone();
            if let Some(auth_header) = headers.get("Authorization") {
                if let Ok(token) = auth_header.to_str() {
                    let user = UserData::authenticate_with_session_token(token).await.map_err(ErrorUnauthorized)?;
                    req.extensions_mut().insert(user);
                    return service.call(req).await.map_err(actix_web::error::ErrorInternalServerError);
                }
            }

            Err(ErrorUnauthorized("Missing or invalid authentication token"))
        })
    }
}
