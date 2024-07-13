use std::time::Duration;
use rdkafka::ClientConfig;

use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;

pub fn create() -> FutureProducer{
    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", "localhost:9092");

    let producer: FutureProducer = config.
        create().
        expect("Producer creation error");

    producer
}

pub async fn produce(future_producer: &FutureProducer, message: &str) {
    let record = FutureRecord::to("test-topic")
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