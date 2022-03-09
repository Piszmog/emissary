use std::future::{ready, Ready};
use std::rc::Rc;

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;

use emissary::Extension;

/// The Logging middleware.
pub struct Extensions {
    pub extensions: Rc<Vec<Box<dyn Extension>>>,
}

impl<S: 'static, B> Transform<S, ServiceRequest> for Extensions
    where
        S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = ExtensionsMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ExtensionsMiddleware {
            service: Rc::new(service),
            extensions: Rc::clone(&self.extensions),
        }))
    }
}

/// The actual logging middleware. Handles the request/response.
pub struct ExtensionsMiddleware<S> {
    service: Rc<S>,
    extensions: Rc<Vec<Box<dyn Extension>>>,
}

impl<S, B> Service<ServiceRequest> for ExtensionsMiddleware<S>
    where
        S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error> + 'static,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        let rc = self.extensions.clone();

        Box::pin(async move {
            rc.iter().for_each(|e| e.modify_request(&mut req));
            let mut res = svc.call(req).await?;
            // self.extensions.iter().for_each(|e| e.modify_response(&mut res));
            Ok(res)
        })
    }
}
