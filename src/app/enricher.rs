#![allow(unused_imports)]

use chrono::Utc;
use netflow_parser::{NetflowPacketResult, NetflowParser};
use rdkafka::message::OwnedMessage;
use ta::db::cidr_lookup::CidrLookup;
use ta::kafka::consumer::start_listener_to_enricher;
use tokio::signal;



#[tokio::main]
async fn main() -> std::io::Result<()> {
    for _ in 0..10{
        tokio::spawn(async move {
            start_listener_to_enricher().await;
        });
    }
    signal::ctrl_c().await.expect("failed to listen for event");
    Ok(())
}