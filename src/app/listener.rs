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


const BUF_SIZE: usize = 2048;


#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let port = args.port;
    {
        let socket = UdpSocket::bind(format!("0.0.0.0:{port}"))?;
    
        let producer = producer::create();

        loop {
            // Receives a single datagram message on the socket. If `buf` is too small to hold
            // the message, it will be cut off.
            // println!(":D");
            let mut buf = [0; BUF_SIZE];


            let (amt, src) = socket.recv_from(&mut buf)?;

            
            let buf: &[u8] = &buf;
            // buf.reverse();
            
            producer::produce_listener_to_enricher(&producer, buf).await;

        }
    } // the socket is closed here
    Ok(())
}
