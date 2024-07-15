#![allow(unused_imports)]

use chrono::Utc;
use netflow_parser::{NetflowPacketResult, NetflowParser};
use rdkafka::message::OwnedMessage;
use ta::db::cidr_lookup::CidrLookup;
use ta::kafka::consumer::start_listener_to_enricher;



#[tokio::main]
async fn main() -> std::io::Result<()> {
    start_listener_to_enricher().await;
    Ok(())
}
