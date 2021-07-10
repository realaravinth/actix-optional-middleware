use std::marker::PhantomData;
/*
* Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
*
* This program is free software: you can redistribute it and/or modify
* it under the terms of the GNU Affero General Public License as
* published by the Free Software Foundation, either version 3 of the
* License, or (at your option) any later version.
*
* This program is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU Affero General Public License for more details.
*
* You should have received a copy of the GNU Affero General Public License
* along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
use std::rc::Rc;

use actix_http::body::AnyBody;
use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::Error;

use futures::future::{ok, Either, LocalBoxFuture, Ready};

pub struct Dummy;

impl<S> Transform<S, ServiceRequest> for Dummy
where
    S: Service<ServiceRequest, Response = ServiceResponse<AnyBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<AnyBody>;
    type Error = Error;
    type Transform = DummyMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(DummyMiddleware { service })
    }
}
pub struct DummyMiddleware<S> {
    service: S,
}

impl<S, Req> Service<Req> for DummyMiddleware<S>
where
    S: Service<Req, Response = ServiceResponse<AnyBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<AnyBody>;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;
    type Error = Error;

    actix_service::forward_ready!(service);

    fn call(&self, req: Req) -> Self::Future {
        println!("executing dummy middleware");
        Either::Left(self.service.call(req))
    }
}

pub enum Group<D, R, Ser>
where
    D: Transform<Ser, ServiceRequest>,
    R: Transform<Ser, ServiceRequest>,
{
    Dummy(Rc<D>),
    Real(Rc<R>),
    Ph(PhantomData<Ser>),
}

// D is dummy
// R is real
pub enum GroupMiddleware<D, R>
where
    //    S: Service<ServiceRequest, Response = ServiceResponse<AnyBody>, Error = Error> + 'static,
    //    S::Future: 'static,
    D: Service<ServiceRequest>,
    R: Service<ServiceRequest>,
{
    //, SS, A: Service<SS> + GetService> {
    Dummy(Rc<D>),
    Real(Rc<R>),
}

impl<D, R, S, DS, RS> Transform<S, ServiceRequest> for Group<D, R, S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<AnyBody>, Error = Error> + 'static,
    D: Transform<S, ServiceRequest, Transform = DS, InitError = (), Error = Error> + 'static,
    R: Transform<S, ServiceRequest, Transform = RS, InitError = (), Error = Error> + 'static,
    DS: Service<ServiceRequest, Error = Error, Response = ServiceResponse> + 'static,
    RS: Service<ServiceRequest, Error = Error, Response = ServiceResponse> + 'static,
    //Combined: Service<ServiceRequest, Response = ServiceResponse<AnyBody>, Error = Error> + 'static,
    //Combined::Future: 'static,
    //Rand: Transform<
    //        Combined,
    //        ServiceRequest,
    //        Response = ServiceResponse,
    //        Transform = Combined,
    //        InitError = (),
    //    > + 'static,
{
    type Response = ServiceResponse<AnyBody>;
    type Error = Error;
    type Transform = GroupMiddleware<DS, RS>;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        match self {
            Self::Real(val) => {
                let val = Rc::clone(&val);
                Box::pin(async move {
                    match val.new_transform(service).await {
                        Ok(val) => Ok(GroupMiddleware::Real(Rc::new(val))),
                        Err(e) => Err(e),
                    }
                })
            }

            Self::Dummy(val) => {
                let val = Rc::clone(&val);
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
    //    S: Service<ServiceRequest, Response = ServiceResponse<AnyBody>, Error = Error> + 'static,
    //    S::Future: 'static,
{
    type Response = ServiceResponse<AnyBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut ::core::task::Context<'_>,
    ) -> ::core::task::Poll<Result<(), Self::Error>> {
        match self {
            Self::Real(val) => val.poll_ready(cx),
            Self::Dummy(val) => val.poll_ready(cx),
            _ => panic!(),
        }
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        println!("executing group middleware");
        match self {
            Self::Real(val) => {
                let val = Rc::clone(val);
                Box::pin(async move { val.call(req).await })
            }
            Self::Dummy(val) => {
                let val = Rc::clone(val);
                Box::pin(async move { val.call(req).await })
            }
            _ => panic!(),
        }
    }
}
