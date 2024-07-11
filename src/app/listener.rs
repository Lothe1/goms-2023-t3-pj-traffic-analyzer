#![allow(unused_imports)]
use clap::Parser;
use ta::cmd::listener::*;
use netflow_parser::{NetflowParser, NetflowPacketResult};
use std::net::UdpSocket;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use rdkafka::util::Timeout;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub mod influx_db;

const BUF_SIZE: usize = 2048;

fn create_producer(bootstrap_server: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", bootstrap_server)
        .set("queue.buffering.max.ms", "0")
        .create().expect("Failed to create client")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let port = args.port;

    {
        let socket = UdpSocket::bind(format!("127.0.0.1:{port}"))?;
        let producer = create_producer("localhost:9092");

        loop {
        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
            let mut buf = [0; BUF_SIZE];
            let (amt, src) = socket.recv_from(&mut buf)?;

            let mut buf: Vec<u8> = buf.to_vec().iter().filter(|&x| *x != 0).map(|x| x.clone()).collect();
            println!("{:?}", String::from_utf8(buf.clone()));
            // Redeclare `buf` as slice of the received data and send reverse data back to origin.
            let buf: &[u8] = &buf;
            // buf.reverse();
            socket.send_to(buf, &src)?;
            producer.send(FutureRecord::<(), _>::to("listener-to-enricher")
                  .payload(buf), Timeout::Never)
                    .await
                    .expect("Failed to produce");
        }
    } // the socket is closed here
    Ok(())
}
