
use std::time::Duration;

use rdkafka::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;

use crate::app::influx_db::CustomMessage;

pub fn create() -> FutureProducer{
    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", "localhost:9092");

    let producer: FutureProducer = config.
        create().
        expect("Producer creation error");

    producer
}


// Do string  for topic listener-to-enricher
pub async fn produce_listener_to_enricher(future_producer: &FutureProducer, message: &[u8]) {
    let record = FutureRecord::to("listener-to-enricher")
        .payload(message)
        .key("Test-key");

    let status_delivery = future_producer
        .send(record, Timeout::After(Duration::from_secs(2)))
        .await;

    match status_delivery{
        Ok(report) => {
            println!("Sent message: {:?}", report);
        },
        Err(e) => {
            println!("Error producing: {:?}", e);
        }
    }
}


// send the network structfor topic enricher-to-tsdb
pub async fn produce_enricher_to_tsb(future_producer: &FutureProducer, message:CustomMessage) {
    let message = serde_json::to_string(&message).expect("Failed to serialize message");

    let record = FutureRecord::to("enricher-to-tsdb")
        .payload(&message)
        .key("Test-key");

    let status_delivery = future_producer
        .send(record, Timeout::After(Duration::from_secs(2)))
        .await;

    match status_delivery{
        Ok(report) => {
            println!("Sent message: {:?}", report);
        },
        Err(e) => {
            println!("Error producing: {:?}", e);
        }
    }
}
