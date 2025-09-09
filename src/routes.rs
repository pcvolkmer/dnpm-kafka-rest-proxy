use crate::sender::DynMtbFileSender;
use crate::AppResponse::{Accepted, InternalServerError, Unauthorized, UnsupportedContentType};
use crate::{auth, CONFIG};
use axum::body::Body;
use axum::extract::Path;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Request};
use axum::middleware::{from_fn, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, post};
use axum::{Extension, Json, Router};
use mv64e_mtb_dto::Mtb;
use tower_http::trace::TraceLayer;

pub async fn handle_delete(
    Path(patient_id): Path<String>,
    Extension(sender): Extension<DynMtbFileSender>,
) -> Response {
    let delete_mtb_file = Mtb::new_with_consent_rejected(&patient_id);
    match sender.send(delete_mtb_file).await {
        Ok(request_id) => Accepted(&request_id).into_response(),
        _ => InternalServerError.into_response(),
    }
}

pub async fn handle_post(
    Extension(sender): Extension<DynMtbFileSender>,
    Json(mtb_file): Json<Mtb>,
) -> Response {
    match sender.send(mtb_file).await {
        Ok(request_id) => Accepted(&request_id).into_response(),
        _ => InternalServerError.into_response(),
    }
}

pub fn routes(sender: DynMtbFileSender) -> Router {
    Router::new()
        .route("/mtb/etl/patient-record", post(handle_post))
        .route(
            "/mtb/etl/patient-record/{patient_id}",
            delete(handle_delete),
        )
        .layer(Extension(sender))
        .layer(from_fn(check_content_type_header))
        .layer(from_fn(check_basic_auth))
        .layer(TraceLayer::new_for_http())
}

async fn check_basic_auth(request: Request<Body>, next: Next) -> Response {
    if let Some(Ok(auth_header)) = request.headers().get(AUTHORIZATION).map(|x| x.to_str())
        && auth::check_basic_auth(auth_header, &CONFIG.token)
    {
        return next.run(request).await;
    }
    log::warn!("Invalid authentication used");
    Unauthorized.into_response()
}

async fn check_content_type_header(request: Request<Body>, next: Next) -> Response {
    match request
        .headers()
        .get(CONTENT_TYPE)
        .map(HeaderValue::as_bytes)
    {
        Some(
            b"application/json"
            | b"application/json; charset=utf-8"
            | b"application/vnd.dnpm.v2.mtb+json",
        ) => next.run(request).await,
        _ => UnsupportedContentType.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sender::MockMtbFileSender;
    use axum::body::Body;
    use axum::http::header::CONTENT_TYPE;
    use axum::http::{Method, Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn should_handle_post_request() {
        let mut sender_mock = MockMtbFileSender::new();

        sender_mock
            .expect_send()
            .withf(|mtb| mtb.patient.id.eq("fae56ea7-24a7-4556-82fb-2b5dde71bb4d"))
            .return_once(move |_| Ok(String::new()));

        let router = routes(Arc::new(sender_mock) as DynMtbFileSender);
        let body = Body::from(include_str!("../test-files/mv64e-mtb-fake-patient.json"));

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mtb/etl/patient-record")
                    .header(AUTHORIZATION, "Basic dG9rZW46dmVyeS1zZWNyZXQ=")
                    .header(CONTENT_TYPE, "application/json")
                    .body(body)
                    .expect("request built"),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn should_handle_delete_request() {
        let mut sender_mock = MockMtbFileSender::new();

        sender_mock
            .expect_send()
            // Expect patient id is set in Kafka record
            .withf(|mtb| mtb.patient.id.eq("fae56ea7-24a7-4556-82fb-2b5dde71bb4d"))
            // Expect no Metadata => no consent in kafka record
            .withf(|mtb| mtb.metadata.is_none())
            .return_once(move |_| Ok(String::new()));

        let router = routes(Arc::new(sender_mock) as DynMtbFileSender);

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/mtb/etl/patient-record/fae56ea7-24a7-4556-82fb-2b5dde71bb4d")
                    .header(AUTHORIZATION, "Basic dG9rZW46dmVyeS1zZWNyZXQ=")
                    .header(CONTENT_TYPE, "application/json")
                    .body(Body::empty())
                    .expect("request built"),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn should_handle_post_request_with_custom_v2_media_type() {
        let mut sender_mock = MockMtbFileSender::new();

        sender_mock
            .expect_send()
            .withf(|mtb| mtb.patient.id.eq("fae56ea7-24a7-4556-82fb-2b5dde71bb4d"))
            .return_once(move |_| Ok(String::new()));

        let router = routes(Arc::new(sender_mock) as DynMtbFileSender);
        let body = Body::from(include_str!("../test-files/mv64e-mtb-fake-patient.json"));

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mtb/etl/patient-record")
                    .header(AUTHORIZATION, "Basic dG9rZW46dmVyeS1zZWNyZXQ=")
                    .header(CONTENT_TYPE, "application/vnd.dnpm.v2.mtb+json")
                    .body(body)
                    .expect("request built"),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn should_not_accept_xml_request() {
        let mut sender_mock = MockMtbFileSender::new();

        sender_mock
            .expect_send()
            .withf(|mtb| mtb.patient.id.eq("fae56ea7-24a7-4556-82fb-2b5dde71bb4d"))
            .return_once(move |_| Ok(String::new()));

        let router = routes(Arc::new(sender_mock) as DynMtbFileSender);
        let body = Body::from("<test>Das ist ein Test</test>");

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mtb/etl/patient-record")
                    .header(AUTHORIZATION, "Basic dG9rZW46dmVyeS1zZWNyZXQ=")
                    .header(CONTENT_TYPE, "application/xml")
                    .body(body)
                    .expect("request built"),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn should_respond_bad_request_if_not_parsable() {
        let mut sender_mock = MockMtbFileSender::new();

        sender_mock
            .expect_send()
            .withf(|mtb| mtb.patient.id.eq("fae56ea7-24a7-4556-82fb-2b5dde71bb4d"))
            .return_once(move |_| Ok(String::new()));

        let router = routes(Arc::new(sender_mock) as DynMtbFileSender);
        let body = Body::from("Das ist kein JSON!");

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mtb/etl/patient-record")
                    .header(AUTHORIZATION, "Basic dG9rZW46dmVyeS1zZWNyZXQ=")
                    .header(CONTENT_TYPE, "application/json")
                    .body(body)
                    .expect("request built"),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn should_respond_bad_request_if_not_processable() {
        let mut sender_mock = MockMtbFileSender::new();

        sender_mock
            .expect_send()
            .withf(|mtb| mtb.patient.id.eq("fae56ea7-24a7-4556-82fb-2b5dde71bb4d"))
            .return_once(move |_| Ok(String::new()));

        let router = routes(Arc::new(sender_mock) as DynMtbFileSender);
        let body = Body::from("{}");

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mtb/etl/patient-record")
                    .header(AUTHORIZATION, "Basic dG9rZW46dmVyeS1zZWNyZXQ=")
                    .header(CONTENT_TYPE, "application/json")
                    .body(body)
                    .expect("request built"),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn should_check_authorization_first() {
        let mut sender_mock = MockMtbFileSender::new();

        sender_mock
            .expect_send()
            .withf(|mtb| mtb.patient.id.eq("fae56ea7-24a7-4556-82fb-2b5dde71bb4d"))
            .return_once(move |_| Ok(String::new()));

        let router = routes(Arc::new(sender_mock) as DynMtbFileSender);
        let body = Body::from("<test>Das ist ein Test</test>");

        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/mtb/etl/patient-record")
                    // No Auth header!
                    .header(CONTENT_TYPE, "application/xml")
                    .body(body)
                    .expect("request built"),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
