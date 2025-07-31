use std::rc::Rc;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use actix_web::error::ErrorUnauthorized;
use actix_web_lab::__reexports::futures_util::future::LocalBoxFuture;

pub struct DDOS;

impl<S, B> Transform<S, ServiceRequest> for DDOS
where
	S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
	B: 'static,
{
	type Response = ServiceResponse<B>;
	type Error = Error;
	type Transform = DDOSMiddleware<S>;
	type InitError = ();
	type Future = std::pin::Pin<Box<dyn Future<Output = Result<Self::Transform, Self::InitError>> + 'static>>;

	fn new_transform(&self, service: S) -> Self::Future {
		Box::pin(async move { Ok(DDOSMiddleware { service: Rc::new(service) }) })
	}
}
pub struct DDOSMiddleware<S> {
	service: Rc<S>,
}


impl<S, B> Service<ServiceRequest> for DDOSMiddleware<S>
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



			Err(ErrorUnauthorized("Missing or invalid authentication token"))
		})

	}
}