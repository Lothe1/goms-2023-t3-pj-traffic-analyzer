use serde_json::json;
use netflow_parser::{NetflowParser, NetflowPacketResult};
use rdkafka::consumer::{Consumer, StreamConsumer};
use uuid::Uuid;
use rdkafka::{ClientConfig, Message};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::fs::File;

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
    // Create the consumer
    let consumer = create_consumer("localhost:9092");

    // Subscribe to our topic
    consumer.subscribe(&["listener-to-enricher"]).unwrap();
    println!("Subscribed! :)");

    let country_cidr_path = "map/ip2country-v4.tsv";
    let as_cidr_path = "map/ip2asn-v4.tsv";

    // Load the CIDR lookup tables
    let cidr_lookup = CidrLookup::new(&country_cidr_path, &as_cidr_path);

    loop {
        let message = consumer.recv().await.expect("Failed to read message").detach();
        let payload = message.payload().unwrap();
        println!("{}", String::from_utf8(payload.to_vec()).unwrap());

        let packet = NetflowParser::default().parse_bytes(&payload).first().unwrap();
        println!("{}", json!(NetflowParser::default().parse_bytes(&payload)).to_string());

        if let NetflowPacketResult::V5(packet) = NetflowParser::default().parse_bytes(&payload).first().unwrap() {
            for flow in &packet.flowsets {
                let src_ip = flow.src_addr.to_string();
                let dst_ip = flow.dst_addr.to_string();

                let src_country = cidr_lookup.lookup_country(&src_ip).unwrap();
                let dst_country = cidr_lookup.lookup_country(&dst_ip).unwrap();
                let src_as = cidr_lookup.lookup_as(&src_ip).unwrap();
                let dst_as = cidr_lookup.lookup_as(&dst_ip).unwrap();

                let enriched_data = json!({
                    "src_ip": src_ip,
                    "dst_ip": dst_ip,
                    "src_country": src_country,
                    "dst_country": dst_country,
                    "src_as": src_as,
                    "dst_as": dst_as,
                    "netflow": flow,
                });

                println!("{}", enriched_data.to_string());
            }
        }
    }
    
    Ok(())
}
