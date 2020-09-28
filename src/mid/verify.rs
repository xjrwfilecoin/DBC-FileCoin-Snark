use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::HeaderValue;
use actix_web::{Error, HttpResponse};
use futures::future::{ok, Either, Ready};
use futures::task::{Context, Poll};
use log::*;

pub struct Verify;

impl<S, B> Transform<S> for Verify
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CheckVerify<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CheckVerify { service })
    }
}

pub struct CheckVerify<S> {
    service: S,
}

impl<S, B> Service for CheckVerify<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        let allow = "/test";
        let path = req.path();

        let token = req
            .headers()
            .get("Authorization")
            .map(|x| check_token(x))
            .unwrap_or(false);

        if token || path == allow {
            Either::Left(self.service.call(req))
        } else {
            Either::Right(ok(req.into_response(HttpResponse::Forbidden().finish().into_body())))
        }
    }
}

fn check_token(token: &HeaderValue) -> bool {
    trace!("checking token {:?}", token.as_bytes());

    true
}
