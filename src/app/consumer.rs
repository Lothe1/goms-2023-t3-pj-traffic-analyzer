#![allow(unused_imports)]

use influxdb::Client;
use rdkafka::consumer::{Consumer, StreamConsumer, CommitMode};
use rdkafka::{ClientConfig, Message};

use crate::app::influx_db::CustomMessage;
use crate::app::influx_db::make_package;
use crate::app::influx_db::IPtype;
use crate::app::influx_db::write_data;
use crate::app::influx_db::create_client;




use serde_json::json;
use netflow_parser::{NetflowParser, NetflowPacketResult};
use uuid::Uuid;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::fs::File;

use chrono::{DateTime, TimeZone, Utc};
use super::cidr_lookup::CidrLookup;

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

                let payload = message.payload().unwrap();
                // println!("{}", String::from_utf8(payload.to_vec()).unwrap());
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
                        let time = Utc.timestamp(packet.header.unix_secs.into(), 0);
                        
                        let package_incoming = make_package(time, &src_ip, &src_as, &src_country, flow.d_octets as i32).await;
                        let package_outgoing = make_package(time, &dst_ip, &dst_as, &dst_country, flow.d_octets as i32).await;



                        let wrapped_msg: CustomMessage = super::producer::make_custom_package(package_incoming, IPtype::Incoming);
                        let wrapped_msg2 = super::producer::make_custom_package(package_outgoing, IPtype::Outgoing);
                        super::producer::produce_enricher_to_tsb(&producer, wrapped_msg).await;
                        super::producer::produce_enricher_to_tsb(&producer, wrapped_msg2).await;

    
                        // let enriched_data = json!({
                        //     "measurement": "netflow",
                        //     "tags": {
                        //         "src_ip": src_ip,
                        //         "dst_ip": dst_ip,
                        //         "src_country": src_country,
                        //         "dst_country": dst_country,
                        //         "src_as": src_as,
                        //         "dst_as": dst_as
                        //     },
                        //     "fields": {
                        //         "packets": flow.d_pkts, // Number of packets
                        //         "bytes": flow.d_octets, // Number of bytes
                        //         "first_switched": flow.first, // Start time of the flow
                        //         "last_switched": flow.last, // End time of the flow
                        //     },
                        //     "time": packet.header.unix_secs // Time of the flow (we'll use the packet's timestamp)
                        // });
                        // println!("{}", enriched_data.to_string());
        
                        // Store the enriched data in InfluxDB
                        // if let Err(e) = store_in_influxdb(&client, &enriched_data).await {
                        //     eprintln!("Failed to store data in InfluxDB: {}", e);
                        // }
                    }
                }


                consumer.commit_message(&message, CommitMode::Async).unwrap();


            }
        }
    }
}

async fn consume_enricher_to_tsdb(consumer:StreamConsumer){
    let client = create_client("db", "ball");
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


                        let _ = write_data(client.clone(), recieved_message.package, recieved_message.iptype).await;

                        
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



