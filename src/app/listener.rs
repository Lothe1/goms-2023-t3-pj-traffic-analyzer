#![allow(unused_imports)]

use clap::Parser;

use netflow_parser::{NetflowParser, NetflowPacketResult};
use ta::kafka::producer;
use ta::cmd::listener::Args;
use std::net::UdpSocket;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use rdkafka::util::Timeout;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::net::{IpAddr};
use tokio::net::lookup_host;


const BUF_SIZE: usize = 2048;


#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let port = args.port;
    let listener_ip = get_listener_ip().await.unwrap();

    {
        let socket = UdpSocket::bind(format!("0.0.0.0:{port}"))?;
        let producer = producer::create();

        loop {
            let mut buf = [0; BUF_SIZE];
            let (amt, src) = socket.recv_from(&mut buf)?;
            let buf: &[u8] = &buf;
            // classify_and_produce(&producer, buf, &listener_ip).await;
            producer::produce_listener_to_enricher(&producer, buf).await;
        }
    }
    Ok(())
}

async fn get_listener_ip() -> Option<IpAddr> {
    let hostnames = lookup_host("localhost").await.ok()?;
    hostnames.map(|x| x.ip()).next()
}