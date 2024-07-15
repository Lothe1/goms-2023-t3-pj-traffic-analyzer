#![allow(unused_imports)]

use influxdb::Client;
use rdkafka::consumer::{Consumer, StreamConsumer, CommitMode};
use rdkafka::{ClientConfig, Message};

use crate::db::influx_db::{CustomMessage, make_package, IPtype, write_data, create_client};
// use crate::app::enricher::enrich_packet;
use serde_json::json;
use netflow_parser::{NetflowParser, NetflowPacketResult};
use uuid::Uuid;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::fs::File;

use chrono::{DateTime, TimeZone, Utc};
use crate::db::cidr_lookup::CidrLookup;

pub async fn start_listener_to_enricher(){
    let consumer: StreamConsumer = create();
    consume_listener_to_enricher(consumer).await;
}

pub async fn start_enricher_to_tsdb(){
    let consumer: StreamConsumer = create();
    consume_enricher_to_tsdb(consumer).await;
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
    //make kafka producer for enricher to tsdb
    
    let producer = super::producer::create();

    // Load the CIDR lookup tables
    let country_cidr_path = "map/ip2country-v4.tsv";
    let as_cidr_path = "map/ip2asn-v4.tsv";
    let cidr_lookup = CidrLookup::new(&country_cidr_path, &as_cidr_path);// Load the CIDR lookup tables
    

    consumer.subscribe
        (
            &["listener-to-enricher"]
        )
        .expect("Can't subscribe to specified topic");

    loop{
        match consumer.recv().await{
            Err(e) => println!("Error receiving message: {:?}", e),
            Ok(message) => {
                println!("Message received: {:?}", message.offset());

                let msg = message.detach();
                let cidr_clone = cidr_lookup.clone();
                tokio::spawn( async move {
                    let my_msg = msg.clone();
                    let payload: Vec<u8> = my_msg.payload().unwrap().iter().cloned().collect();
                    enrich_packet(payload.clone(), cidr_clone)
                    
                }
                );
                consumer.commit_message(&message, CommitMode::Async).unwrap();
            }
        }
    }
}

async fn consume_enricher_to_tsdb(consumer:StreamConsumer){
    let client = Client::new("http://localhost:8086", "db")
        .with_token("ball");

    consumer.subscribe
    (
        &["enricher-to-tsdb"]
    )
        .expect("Can't subscribe to specified topic");

    loop{
        match consumer.recv().await{
            Err(e) => println!("Error receiving message: {:?}", e),
            Ok(message) => {
                match message.payload_view::<str>(){
                    None => println!("NO message"),
                    Some(Ok(s)) => {
                        let recieved_message: CustomMessage = serde_json::from_str(s).expect("Failed to deserialize message");
                        println!("Message: {:?} recieved", recieved_message);


                        let write_res = write_data(client.clone(), recieved_message.package, recieved_message.iptype).await;

                        match write_res{
                            Ok(_) => println!("Data written to db"),
                            Err(e) => println!("Error writing to db: {:?}", e)
                        }
                        
                    },
                    Some(Err(e)) => {
                        println!("Error unpacking message: {:?}", e);
                    }
                }
                consumer.commit_message(&message, CommitMode::Async).unwrap();
            }
        }
    }
}




pub async fn enrich_packet(payload: Vec<u8>, cidr_lookup: CidrLookup) {

    if let NetflowPacketResult::V5(packet) = NetflowParser::default().parse_bytes(&payload).first().unwrap() {
        println!("Begin for flow in packet.flowsets");
        for flow in &packet.flowsets {

            let src_ip = flow.src_addr.to_string();
            let dst_ip = flow.dst_addr.to_string();
            let src_country = cidr_lookup.lookup_country(&src_ip).unwrap();
            let dst_country = cidr_lookup.lookup_country(&dst_ip).unwrap();
            let src_as = cidr_lookup.lookup_as(&src_ip).unwrap();
            let dst_as = cidr_lookup.lookup_as(&dst_ip).unwrap();

            
            let time = Utc::now();

            // let package_incoming = make_package(time, &src_ip, &src_as, &src_country, flow.d_octets as i32).await;
            // let package_outgoing = make_package(time, &dst_ip, &dst_as, &dst_country, flow.d_octets as i32).await;



            // let wrapped_msg: CustomMessage = super::producer::make_custom_package(package_incoming, IPtype::Incoming);
            // let wrapped_msg2 = super::producer::make_custom_package(package_outgoing, IPtype::Outgoing);
            // let _ = super::producer::produce_enricher_to_tsb(&producer, wrapped_msg).await;
            // let _ = super::producer::produce_enricher_to_tsb(&producer, wrapped_msg2).await;


        }
    }
}