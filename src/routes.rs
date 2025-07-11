use crate::sender::DynMtbFileSender;
use crate::AppResponse::{Accepted, InternalServerError};
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, post};
use axum::{Extension, Json, Router};
use mv64e_mtb_dto::Mtb;

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
                    .header(CONTENT_TYPE, "application/json")
                    .body(Body::empty())
                    .expect("request built"),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }
}
