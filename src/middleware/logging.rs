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

use crate::config;
use crate::utils::Plain;

/// The Logging middleware.
pub struct Logging {
    pub mode: config::LoggingMode,
    pub json: Json,
    pub plain: Plain,
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
        ready(Ok(LoggingMiddleware {
            service: Rc::new(service),
            mode: self.mode.clone(),
            json: self.json.clone(),
            plain: self.plain.clone(),
        }))
    }
}

/// The actual logging middleware. Handles the request/response.
pub struct LoggingMiddleware<S> {
    service: Rc<S>,
    mode: config::LoggingMode,
    json: Json,
    plain: Plain,
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
        let mode = self.mode.clone();
        let json = self.json.clone();
        let plain = self.plain.clone();
        let svc = self.service.clone();

        Box::pin(async move {
            let head = req.head().clone();

            let start = Instant::now();
            let res = svc.call(req).await?;
            let end = Instant::now();

            let context = Context {
                start,
                end,
                req_head: head,
                res_head: res.response().head().clone(),
            };

            log(mode, json, plain, context);

            Ok(res)
        })
    }
}

fn log(mode: config::LoggingMode, mut json: Json, mut plain: Plain, context: Context) {
    match mode {
        config::LoggingMode::Json => {
            populate_json(&mut json, &context);
            info!("{}", json);
        }
        config::LoggingMode::Plain => {
            populate_plain(&mut plain, context);
            info!("{}", plain);
        }
    }
}

/// The context of the request/response.
#[derive(Debug)]
struct Context {
    /// The start time of the request.
    start: Instant,
    /// The end time of the request.
    end: Instant,
    /// The request head.
    req_head: RequestHead,
    /// The response head.
    res_head: ResponseHead,
}

/// Populate the JSON with the context information. Some of the fields may have values that need to
/// be calculated at runtime.
fn populate_json(value: &mut Json, context: &Context) {
    // create the regexes once
    lazy_static! {
        static ref RE_REQ_HEADER: Regex = Regex::new(r"%\{REQUEST_HEADER\((.*)\)}" ).unwrap();
        static ref RE_RES_HEADER: Regex = Regex::new(r"%\{RESPONSE_HEADER\((.*)\)}" ).unwrap();
    }

    match value {
        Json::Array(arr) => {
            for v in arr {
                populate_json(v, &context);
            }
        }
        Json::Object(obj) => {
            for (_, v) in obj {
                populate_json(v, &context);
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
                "%{STATUS_CODE}" => *value = Json::Number(Number::from(context.res_head.status.as_u16())),
                _ => {
                    // To handle HTTP headers, we need to use the regexes. Since the key cannot be
                    // known ahead of time.
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

fn populate_plain(value: &mut Plain, context: Context) {
    // create the regexes once
    lazy_static! {
        static ref RE_REQ_HEADER: Regex = Regex::new(r"%\{REQUEST_HEADER\((.*)\)}" ).unwrap();
        static ref RE_RES_HEADER: Regex = Regex::new(r"%\{RESPONSE_HEADER\((.*)\)}" ).unwrap();
    }

    value.data.values_mut().for_each(|s| {
        match s.as_str() {
            "%{UUID()}" => *s = uuid(),
            "%{DURATION(NS)}" => *s = duration(context.start, context.end, "ns").unwrap(),
            "%{DURATION(MS)}" => *s = duration(context.start, context.end, "ms").unwrap(),
            "%{DURATION(S)}" => *s = duration(context.start, context.end, "s").unwrap(),
            "%{METHOD}" => *s = context.req_head.method.to_string(),
            "%{URI}" => *s = context.req_head.uri.to_string(),
            "%{STATUS_CODE}" => *s = context.res_head.status.as_u16().to_string(),
            _ => {
                // To handle HTTP headers, we need to use the regexes. Since the key cannot be
                // known ahead of time.
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
    })
}

/// Generate a UUID.
fn uuid() -> String {
    Uuid::new_v4().to_string()
}

/// Returns the duration between two instants. The unit is specified by the parameter `duration_type`.
fn duration(start: Instant, end: Instant, duration_type: &str) -> Result<String, String> {
    match duration_type {
        // Nanoseconds
        "ns" => Ok(end.duration_since(start).as_nanos().to_string()),
        // Milliseconds
        "ms" => Ok(end.duration_since(start).as_millis().to_string()),
        // Seconds
        "s" => Ok(end.duration_since(start).as_secs().to_string()),
        // Not supported
        _ => Err(format!("{} is not a valid duration type", duration_type)),
    }
}

/// Returns the value of the header specified by the parameter `key`.
fn header(headers: &HeaderMap, key: &str) -> String {
    headers.get(key)
        .map(|header| header.to_str().unwrap().to_string())
        // If the header is not found, return an empty string
        .unwrap_or_else(|| "".to_string())
}
