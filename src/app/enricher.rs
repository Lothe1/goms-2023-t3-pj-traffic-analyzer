#![allow(unused_imports)]



use serde_json::json;

use netflow_parser::{NetflowParser, NetflowPacketResult};
use rdkafka::consumer::{Consumer, StreamConsumer};

use uuid::Uuid;
use rdkafka::{ClientConfig, Message};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::fs::File;
const BUF_SIZE: usize = 2048;

use ta::app::consumer::start_listener_to_enricher;



#[tokio::main]
async fn main() -> std::io::Result<()> {

    start_listener_to_enricher().await;
    Ok(())
}
