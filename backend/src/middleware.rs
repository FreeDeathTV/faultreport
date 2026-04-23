use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures::future::LocalBoxFuture;
use futures::FutureExt;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

pub mod auth;

pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + Send + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersMiddleware<S>;
    type Future = Pin<Box<dyn futures::Future<Output = Result<Self::Transform, Self::InitError>> + Send>>;

    fn new_transform(&self, service: S) -> Self::Future {
        Box::pin(async move { Ok(SecurityHeadersMiddleware { service: Rc::new(service) }) })
    }
}

pub struct SecurityHeadersMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        async move {
            let mut res = service.call(req).await?;
            use actix_web::http::header::{CONTENT_SECURITY_POLICY, X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS, REFERRER_POLICY};
            res.headers_mut().insert(CONTENT_SECURITY_POLICY, "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'".parse().unwrap());
            res.headers_mut().insert(X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
            res.headers_mut().insert(X_FRAME_OPTIONS, "DENY".parse().unwrap());
            res.headers_mut().insert(REFERRER_POLICY, "strict-origin-when-cross-origin".parse().unwrap());

            Ok(res)
        }.boxed_local()
    }
}
