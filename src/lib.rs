/*
* Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
*
* Use of this source code is governed by the Apache 2.0 and/or the MIT
* License.
*/
//! ```rust
//! use std::rc::Rc;
//!
//! use actix_optional_middleware::{Group, Dummy};
//! use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
//! use actix_web::middleware::DefaultHeaders;
//! use actix_web::{web, body::BoxBody, App, Error, HttpServer, Responder, get};
//!
//!#[get("/test", wrap = "get_group_middleware()")]
//! async fn h1() -> impl Responder {
//!     "Handler 1"
//! }
//!
//! // flip this value to see dummy in action
//! const ACTIVE: bool = true;
//!
//! fn get_group_middleware<S>() -> Group<Dummy, DefaultHeaders, S>
//! where
//!     S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
//! {
//!     if ACTIVE {
//!         Group::Real(Rc::new(DefaultHeaders::new()
//!                .header("Permissions-Policy", "interest-cohort=()"
//!         )))
//!     } else {
//!         Group::default()
//!     }
//! }
//! ```

#![allow(clippy::type_complexity)]
use std::marker::PhantomData;
use std::rc::Rc;

use actix_http::body::BoxBody;
use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::Error;

use futures::future::{ok, Either, LocalBoxFuture, Ready};

/// Dummy Middleware: it simply forwards the request without operating on it
pub struct Dummy;

impl<S> Transform<S, ServiceRequest> for Dummy
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = DummyMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(DummyMiddleware { service })
    }
}

/// Dummy Middleware: it simply forwards the request without operating on it
pub struct DummyMiddleware<S> {
    service: S,
}

impl<S, Req> Service<Req> for DummyMiddleware<S>
where
    S: Service<Req, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;
    type Error = Error;

    actix_service::forward_ready!(service);

    fn call(&self, req: Req) -> Self::Future {
        Either::Left(self.service.call(req))
    }
}

/// Collection datatype that encapsulates dummy and real middlewares
///
/// The appropriate middleware is executed based on the variant chosen
pub enum Group<D, R, Ser>
where
    D: Transform<Ser, ServiceRequest>,
    R: Transform<Ser, ServiceRequest>,
{
    Dummy(Rc<D>),
    Real(Rc<R>),
    __(PhantomData<Ser>),
}

impl<R, Ser> Default for Group<Dummy, R, Ser>
where
    R: Transform<Ser, ServiceRequest>,
    Ser: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    fn default() -> Self {
        Self::Dummy(Rc::new(Dummy))
    }
}

// D is dummy
// R is real
pub enum GroupMiddleware<D, R>
where
    D: Service<ServiceRequest>,
    R: Service<ServiceRequest>,
{
    Dummy(Rc<D>),
    Real(Rc<R>),
}

impl<D, R, S, DS, RS> Transform<S, ServiceRequest> for Group<D, R, S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    D: Transform<S, ServiceRequest, Transform = DS, InitError = (), Error = Error> + 'static,
    R: Transform<S, ServiceRequest, Transform = RS, InitError = (), Error = Error> + 'static,
    DS: Service<ServiceRequest, Error = Error, Response = ServiceResponse> + 'static,
    RS: Service<ServiceRequest, Error = Error, Response = ServiceResponse> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = GroupMiddleware<DS, RS>;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        match self {
            Self::Real(val) => {
                let val = Rc::clone(val);
                Box::pin(async move {
                    match val.new_transform(service).await {
                        Ok(val) => Ok(GroupMiddleware::Real(Rc::new(val))),
                        Err(e) => Err(e),
                    }
                })
            }

            Self::Dummy(val) => {
                let val = Rc::clone(val);
                Box::pin(async move {
                    match val.new_transform(service).await {
                        Ok(val) => Ok(GroupMiddleware::Dummy(Rc::new(val))),
                        Err(e) => Err(e),
                    }
                })
            }

            _ => panic!(),
        }
    }
}

impl<D, R> Service<ServiceRequest> for GroupMiddleware<D, R>
where
    D: Service<ServiceRequest, Error = Error, Response = ServiceResponse> + 'static,
    R: Service<ServiceRequest, Error = Error, Response = ServiceResponse> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut ::core::task::Context<'_>,
    ) -> ::core::task::Poll<Result<(), Self::Error>> {
        match self {
            Self::Real(val) => val.poll_ready(cx),
            Self::Dummy(val) => val.poll_ready(cx),
            #[allow(unreachable_patterns)]
            _ => panic!(),
        }
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        match self {
            Self::Real(val) => {
                let val = Rc::clone(val);
                Box::pin(async move { val.call(req).await })
            }
            Self::Dummy(val) => {
                let val = Rc::clone(val);
                Box::pin(async move { val.call(req).await })
            }
            #[allow(unreachable_patterns)]
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::middleware::DefaultHeaders;
    use actix_web::Responder;
    use actix_web::{http, test, App};

    const ENABLED: &str = "/enabled";
    const DISABLED: &str = "/disabled";
    const DEFAULT_HEADER: (&str, &str) = ("Permissions-Policy", "interest-cohort=()");

    #[my_codegen::get(path = "ENABLED", wrap = "get_enabled()")]
    async fn enabled() -> impl Responder {
        "Enabled"
    }

    #[my_codegen::get(path = "DISABLED", wrap = "get_disabled()")]
    async fn disabled() -> impl Responder {
        "Disabled"
    }

    fn get_enabled<S>() -> Group<Dummy, DefaultHeaders, S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    {
        get_group_middleware(true)
    }

    fn get_disabled<S>() -> Group<Dummy, DefaultHeaders, S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    {
        get_group_middleware(false)
    }

    fn get_group_middleware<S>(active: bool) -> Group<Dummy, DefaultHeaders, S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    {
        if active {
            Group::Real(Rc::new(
                DefaultHeaders::new().header(DEFAULT_HEADER.0, DEFAULT_HEADER.1),
            ))
        } else {
            Group::default()
        }
    }

    fn service(cfg: &mut actix_web::web::ServiceConfig) {
        cfg.service(enabled);
        cfg.service(disabled);
    }
    #[actix_rt::test]
    async fn test() {
        let app = test::init_service(App::new().configure(service)).await;

        // test enabled
        let resp =
            test::call_service(&app, test::TestRequest::get().uri(ENABLED).to_request()).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        assert!(resp
            .headers()
            .iter()
            .any(|(k, v)| k == DEFAULT_HEADER.0 && v == DEFAULT_HEADER.1));

        // test disabled
        let resp =
            test::call_service(&app, test::TestRequest::get().uri(DISABLED).to_request()).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        assert!(!resp
            .headers()
            .iter()
            .any(|(k, v)| k == DEFAULT_HEADER.0 && v == DEFAULT_HEADER.1))
    }
}
