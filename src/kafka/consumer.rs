#![allow(unused_imports)]

use influxdb::Client;
use rdkafka::consumer::{Consumer, StreamConsumer, CommitMode};
use rdkafka::producer::FutureRecord;
use rdkafka::util::Timeout;
use rdkafka::{ClientConfig, Message};

use crate::db::influx_db::{CustomMessage, make_package, write_data, create_client};
use crate::db::ip_lookup::IPtype;
use crate::process::enricher::enrich_packet;
// use crate::app::enricher::enrich_packet;
use serde_json::json;
use netflow_parser::{NetflowParser, NetflowPacketResult};
use uuid::Uuid;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::fs::File;
use tokio::net::lookup_host;
use std::net::IpAddr;

use chrono::{DateTime, TimeZone, Utc};
use crate::db::cidr_lookup::CidrLookup;

pub async fn start_listener_to_enricher(){
    let consumer: StreamConsumer = create();
    // let listener_ip = get_listener_ip().await.unwrap(); // Retrieve listener IP here
    consume_listener_to_enricher(consumer).await; // Pass listener_ip to function
}



pub fn create() ->StreamConsumer {
    let mut config = ClientConfig::new();

    config.set("bootstrap.servers", "localhost:9092");
    config.set("auto.offset.reset", "earliest");
    config.set("group.id", "test-group");
    config.set("socket.timeout.ms", "4000");
    let consumer: StreamConsumer =
        config.create()
            .expect("Consumer creation failed");

    consumer
}




async fn consume_listener_to_enricher(consumer:StreamConsumer){
    // Make kafka producer for enricher to tsdb
    
    let producer = super::producer::create();

    // Load the CIDR lookup tables
    let country_cidr_path = "map/ip2country-v4.tsv";
    let as_cidr_path = "map/ip2asn-v4.tsv";
    let cidr_lookup = CidrLookup::new(&country_cidr_path, &as_cidr_path);

    consumer.subscribe(&["listener-to-enricher"]).expect("Can't subscribe to specified topic");

    loop {
        match consumer.recv().await {
            Err(e) => println!("Error receiving message: {:?}", e),
            Ok(message) => {
                let msg = message.detach();
                let cidr_clone = cidr_lookup.clone();
                let this_producer = producer.clone();
                
                let my_msg = msg.clone();
                let payload: Vec<u8> = my_msg.payload().unwrap().iter().cloned().collect();
                let packets = enrich_packet(payload.clone(), cidr_clone).await;
                for packet in packets {
                    this_producer.send(FutureRecord::<(), _>::to("enricher-to-tsdb")
                        .payload(&packet), Timeout::Never)
                        .await
                        .expect("Failed to produce");
                }
                // println!("Sent all data!");
                
                consumer.commit_message(&message, CommitMode::Async).unwrap();
            }
        }
    }
}


async fn get_listener_ip() -> Option<IpAddr> {
    let hostnames = lookup_host("localhost").await.ok()?;
    hostnames.map(|x| x.ip()).next()
}
