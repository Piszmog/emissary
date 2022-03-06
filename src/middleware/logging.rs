use std::future::{ready, Ready};
use std::rc::Rc;
use std::time::Instant;

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use actix_web::dev::{RequestHead, ResponseHead};
use actix_web::http::header::HeaderMap;
use clap::lazy_static::lazy_static;
use futures_util::future::LocalBoxFuture;
use regex::Regex;
use serde_json::{Number, Value as Json};
use tracing::info;
use uuid::Uuid;

pub struct Logging {
    pub json: Json,
}

impl<S: 'static, B> Transform<S, ServiceRequest> for Logging
    where
        S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = LoggingMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoggingMiddleware { service: Rc::new(service), json: self.json.clone() }))
    }
}

pub struct LoggingMiddleware<S> {
    service: Rc<S>,
    json: Json,
}

impl<S, B> Service<ServiceRequest> for LoggingMiddleware<S>
    where
        S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error> + 'static,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let mut json = self.json.clone();
        let svc = self.service.clone();

        Box::pin(async move {
            let head = req.head().clone();

            let start = Instant::now();
            let res = svc.call(req).await?;
            let end = Instant::now();

            populate(&mut json, &Context {
                start,
                end,
                req_head: head,
                status_code: res.status().as_u16(),
                res_head: res.response().head().clone(),
            });

            info!("{}", json);

            Ok(res)
        })
    }
}


struct Context {
    pub start: Instant,
    pub end: Instant,
    pub req_head: RequestHead,
    pub status_code: u16,
    pub res_head: ResponseHead,
}

fn populate(value: &mut Json, context: &Context) {
    lazy_static! {
        static ref RE_REQ_HEADER: Regex = Regex::new(r"%\{REQUEST_HEADER\((.*)\)}" ).unwrap();
        static ref RE_RES_HEADER: Regex = Regex::new(r"%\{RESPONSE_HEADER\((.*)\)}" ).unwrap();
    }

    match value {
        Json::Array(arr) => {
            for v in arr {
                populate(v, &context);
            }
        }
        Json::Object(obj) => {
            for (_, v) in obj {
                populate(v, &context);
            }
        }
        Json::String(s) => {
            match s.as_str() {
                "%{UUID()}" => *s = uuid(),
                "%{DURATION(NS)}" => *s = duration(context.start, context.end, "ns").unwrap(),
                "%{DURATION(MS)}" => *s = duration(context.start, context.end, "ms").unwrap(),
                "%{DURATION(S)}" => *s = duration(context.start, context.end, "s").unwrap(),
                "%{METHOD}" => *s = context.req_head.method.to_string(),
                "%{URI}" => *s = context.req_head.uri.to_string(),
                "%{STATUS_CODE}" => *value = Json::Number(Number::from(context.status_code)),
                _ => {
                    match RE_REQ_HEADER.captures(s) {
                        Some(cap) => {
                            *s = header(context.req_head.headers(), &cap[1]);
                        }
                        None => {}
                    };
                    match RE_RES_HEADER.captures(s) {
                        Some(cap) => {
                            *s = header(context.res_head.headers(), &cap[1]);
                        }
                        None => {}
                    };
                }
            }
        }
        _ => (),
    }
}

fn uuid() -> String {
    Uuid::new_v4().to_string()
}

fn duration(start: Instant, end: Instant, duration_type: &str) -> Result<String, String> {
    match duration_type {
        "ns" => Ok(end.duration_since(start).as_nanos().to_string()),
        "ms" => Ok(end.duration_since(start).as_millis().to_string()),
        "s" => Ok(end.duration_since(start).as_secs().to_string()),
        _ => Err(format!("{} is not a valid duration type", duration_type)),
    }
}

fn header(headers: &HeaderMap, key: &str) -> String {
    headers.get(key)
        .map(|header| header.to_str().unwrap().to_string())
        .unwrap_or_else(|| "".to_string())
}