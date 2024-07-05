use serde_json::json;
use netflow_parser::{NetflowParser, NetflowPacketResult};
use rdkafka::consumer::{Consumer, StreamConsumer};
use uuid::Uuid;
use rdkafka::{ClientConfig, Message};

mod cidr_lookup;
use cidr_lookup::CidrLookup;

const BUF_SIZE: usize = 2048;

fn create_consumer(bootstrap_server: &str) -> StreamConsumer {
    ClientConfig::new()
        .set("bootstrap.servers", bootstrap_server)
        .set("enable.partition.eof", "false")
        // We'll give each session its own (unique) consumer group id,
        // so that each session will receive all messages
        .set("group.id", format!("chat-{}", Uuid::new_v4()))
        .create()
        .expect("Failed to create client")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // create the consumer
    let consumer = create_consumer("localhost:9092");

    // subscribe to our topic
    consumer.subscribe(&["listener-to-enricher"]).unwrap();
    println!("Subscribed! :)");
    loop {
        let message = consumer.recv();
        let message  = message.await.expect("Failed to read message").detach();
        let payload = message.payload().unwrap();
        println!("{}", String::from_utf8(payload.to_vec()).unwrap());

        let packet = NetflowParser::default()
        .parse_bytes(&payload)
        .first()
        .unwrap();
        println!("{}", json!(NetflowParser::default().parse_bytes(&payload)).to_string());
        }
    Ok(())
}
