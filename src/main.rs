use axum::body::Body;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Request, StatusCode};
use axum::middleware::{from_fn, Next};
use axum::response::{IntoResponse, Response};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
#[cfg(debug_assertions)]
use tower_http::trace::TraceLayer;

use crate::cli::Cli;
use crate::routes::routes;
use crate::sender::DefaultMtbFileSender;
use crate::AppResponse::{Accepted, Unauthorized, UnsupportedContentType};

mod auth;
mod cli;
mod routes;
mod sender;

#[derive(Serialize, Deserialize)]
struct RecordKey {
    #[serde(rename = "pid")]
    patient_id: String,
}

enum AppResponse<'a> {
    Accepted(&'a str),
    Unauthorized,
    InternalServerError,
    UnsupportedContentType,
}

#[allow(clippy::expect_used)]
impl IntoResponse for AppResponse<'_> {
    fn into_response(self) -> Response {
        match self {
            UnsupportedContentType => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "This application accepts DNPM data model version 2.1 with content type 'application/json'"
            ).into_response(),
            _ => match self {
                Accepted(request_id) => Response::builder()
                    .status(StatusCode::ACCEPTED)
                    .header("X-Request-Id", request_id),
                Unauthorized => Response::builder().status(StatusCode::UNAUTHORIZED),
                _ => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR),
            }
                .body(Body::empty()).expect("response built"),
        }
    }
}

static CONFIG: LazyLock<Cli> = LazyLock::new(Cli::parse);

#[tokio::main]
async fn main() -> Result<(), ()> {
    #[cfg(debug_assertions)]
    {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }

    let sender = Arc::new(DefaultMtbFileSender::new(
        &CONFIG.topic,
        &CONFIG.bootstrap_server,
    )?);

    let routes = routes(sender)
        .layer(from_fn(check_content_type_header))
        .layer(from_fn(check_basic_auth));

    #[cfg(debug_assertions)]
    let routes = routes.layer(TraceLayer::new_for_http());

    match tokio::net::TcpListener::bind(&CONFIG.listen).await {
        Ok(listener) => {
            log::info!("Starting application listening on '{}'", CONFIG.listen);
            if let Err(err) = axum::serve(listener, routes).await {
                log::error!("Error starting application: {err}");
            }
        }
        Err(err) => log::error!("Error listening on '{}': {}", CONFIG.listen, err),
    }

    Ok(())
}

async fn check_content_type_header(request: Request<Body>, next: Next) -> Response {
    match request
        .headers()
        .get(CONTENT_TYPE)
        .map(HeaderValue::as_bytes)
    {
        Some(b"application/json" | b"application/json; charset=utf-8") => next.run(request).await,
        _ => UnsupportedContentType.into_response(),
    }
}

async fn check_basic_auth(request: Request<Body>, next: Next) -> Response {
    if let Some(Ok(auth_header)) = request.headers().get(AUTHORIZATION).map(|x| x.to_str()) {
        if auth::check_basic_auth(auth_header, &CONFIG.token) {
            return next.run(request).await;
        }
    }
    Unauthorized.into_response()
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use uuid::Uuid;

    use crate::AppResponse::{Accepted, InternalServerError};

    #[test]
    fn should_return_success_response() {
        let response = Accepted(&Uuid::new_v4().to_string()).into_response();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        assert!(response.headers().contains_key("x-request-id"));
    }

    #[test]
    fn should_return_error_response() {
        let response = InternalServerError.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(!response.headers().contains_key("x-request-id"));
    }
}
