use async_trait::async_trait;
use mv64e_mtb_dto::Mtb;
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

#[cfg(test)]
use mockall::automock;

use crate::RecordKey;

pub type DynMtbFileSender = Arc<dyn MtbFileSender + Send + Sync>;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait MtbFileSender {
    async fn send(&self, mtb: Mtb) -> Result<String, ()>;
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct DefaultMtbFileSender {
    topic: String,
    producer: FutureProducer,
}

impl DefaultMtbFileSender {
    pub fn new(topic: &str, bootstrap_server: &str) -> Result<Self, ()> {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_server)
            .set("message.timeout.ms", "5000")
            .create::<FutureProducer>()
            .map_err(|_| ())?;

        Ok(Self {
            topic: topic.to_string(),
            producer,
        })
    }
}

#[async_trait]
impl MtbFileSender for DefaultMtbFileSender {
    async fn send(&self, mtb: Mtb) -> Result<String, ()> {
        let request_id = Uuid::new_v4();

        let record_key = RecordKey {
            patient_id: mtb.patient.id.to_string(),
        };

        let record_headers = OwnedHeaders::default().insert(Header {
            key: "requestId",
            value: Some(&request_id.to_string()),
        });

        let record_key = serde_json::to_string(&record_key).map_err(|_| ())?;

        match serde_json::to_string(&mtb) {
            Ok(json) => {
                self.producer
                    .send(
                        FutureRecord::to(&self.topic)
                            .key(&record_key)
                            .headers(record_headers)
                            .payload(&json),
                        Duration::from_secs(1),
                    )
                    .await
                    .map_err(|_| ())
                    .map(|_| ())?;
                Ok(request_id.to_string())
            }
            Err(_) => Err(()),
        }
    }
}
