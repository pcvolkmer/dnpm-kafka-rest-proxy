use axum::body::Body;
use axum::http::header::WWW_AUTHENTICATE;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use rdkafka::producer::FutureProducer;
use rdkafka::ClientConfig;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};

#[cfg(not(test))]
use clap::Parser;

use crate::cli::Cli;
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
                Unauthorized => Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header(WWW_AUTHENTICATE, "Basic realm=\"DNPM Kafka Rest Proxy Realm\""),
                _ => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR),
            }
                .body(Body::empty()).expect("response built"),
        }
    }
}

#[cfg(not(test))]
static CONFIG: LazyLock<Cli> = LazyLock::new(Cli::parse);

#[tokio::main]
async fn main() -> Result<(), ()> {
    #[cfg(debug_assertions)]
    {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }

    #[cfg(not(debug_assertions))]
    {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    let producer = if CONFIG.ssl_cert_file.is_some() || CONFIG.ssl_key_file.is_some() {
        // Use SSL
        ClientConfig::new()
            .set("bootstrap.servers", &CONFIG.bootstrap_server)
            .set("message.timeout.ms", "5000")
            .set("security.protocol", "ssl")
            .set(
                "ssl.ca.location",
                CONFIG.ssl_ca_file.clone().unwrap_or_default(),
            )
            .set(
                "ssl.certificate.location",
                CONFIG.ssl_cert_file.clone().unwrap_or_default(),
            )
            .set(
                "ssl.key.location",
                CONFIG.ssl_key_file.clone().unwrap_or_default(),
            )
            .create::<FutureProducer>()
            .map_err(|_| ())?
    } else {
        // Plain
        ClientConfig::new()
            .set("bootstrap.servers", &CONFIG.bootstrap_server)
            .set("message.timeout.ms", "5000")
            .create::<FutureProducer>()
            .map_err(|_| ())?
    };

    let sender = Arc::new(DefaultMtbFileSender::new(&CONFIG.topic, producer));

    match tokio::net::TcpListener::bind(&CONFIG.listen).await {
        Ok(listener) => {
            log::info!("Starting application listening on '{}'", CONFIG.listen);
            if let Err(err) = axum::serve(listener, routes::routes(sender)).await {
                log::error!("Error starting application: {err}");
            }
        }
        Err(err) => log::error!("Error listening on '{}': {}", CONFIG.listen, err),
    }

    Ok(())
}

// Test Configuration
#[cfg(test)]
static CONFIG: LazyLock<Cli> = LazyLock::new(|| Cli {
    bootstrap_server: "localhost:9094".to_string(),
    topic: "test-topic".to_string(),
    // Basic dG9rZW46dmVyeS1zZWNyZXQ=
    token: "$2y$05$LIIFF4Rbi3iRVA4UIqxzPeTJ0NOn/cV2hDnSKFftAMzbEZRa42xSG".to_string(),
    listen: "0.0.0.0:3000".to_string(),
    ssl_ca_file: None,
    ssl_cert_file: None,
    ssl_key_file: None,
});

#[cfg(test)]
mod tests {
    use axum::http::header::WWW_AUTHENTICATE;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use uuid::Uuid;

    use crate::AppResponse::{Accepted, InternalServerError, Unauthorized};

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

    #[test]
    fn should_return_unauthorized_response() {
        let response = Unauthorized.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(response.headers().contains_key(WWW_AUTHENTICATE));
    }
}
