#![allow(unused_imports)]

use clap::Parser;

use netflow_parser::static_versions::v5::FlowSet;
use netflow_parser::{NetflowParser, NetflowPacketResult};
use ta::kafka::producer;
use ta::cmd::listener::Args;
use std::net::{UdpSocket,IpAddr};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use rdkafka::util::Timeout;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use ta::db::ip_lookup::IPtype;
use serde_json::{json, Value};
use chrono::Utc;
use tokio::net::lookup_host;
use get_if_addrs::get_if_addrs;

const BUF_SIZE: usize = 2048;


#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let port = args.port;

    // Attempt to get the listener IP
    let listener_ip = match get_listener_ip().await {
        Some(ip) => ip,
        None => {
            eprintln!("Error: Failed to retrieve listener IP.");
            return Ok(());
        }
    };

    let socket = std::net::UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    let producer = producer::create();

    loop {
        let mut buf = [0; BUF_SIZE];
        let (amt, src) = socket.recv_from(&mut buf)?;
        let buf: &[u8] = &buf;
        classify_and_produce(&producer, buf, &listener_ip).await;
    }

    Ok(())
}

async fn get_listener_ip() -> Option<IpAddr> {
    // Retrieve all network interfaces
    if let Ok(if_addrs) = get_if_addrs() {
        // Filter for IPv4 addresses within the 192.168.x.x range
        for iface in if_addrs {
            if let addr = iface.ip() {
                if let IpAddr::V4(ipv4_addr) = addr {
                    let octets = ipv4_addr.octets();
                    if octets[0] == 192 && octets[1] == 168 {
                        return Some(IpAddr::V4(ipv4_addr));
                    }
                }
            }
        }
    }

    None
}

async fn classify_and_produce(producer: &FutureProducer, buf: &[u8], listener_ip: &IpAddr) {
    if let Some(NetflowPacketResult::V5(packet)) = NetflowParser::default().parse_bytes(buf).first() {
        for flow in &packet.flowsets {
            let src_ip = flow.src_addr.to_string();
            let dst_ip = flow.dst_addr.to_string();

            let packet_type = if &flow.src_addr == listener_ip {
                IPtype::Outgoing
            } else if &flow.dst_addr == listener_ip {
                IPtype::Incoming
            } else {
                continue; // Skip packets that are neither incoming nor outgoing
            };

            let enriched_data = enrich_flow_data(flow, &src_ip, &dst_ip, packet_type);
            let buf = serde_json::to_vec(&enriched_data).unwrap();
            producer::produce_listener_to_enricher(producer, &buf).await;
        }
    }
}

fn enrich_flow_data(flow: &FlowSet, src_ip: &str, dst_ip: &str, packet_type: IPtype) -> Value {
    json!({
        "measurement": "netflow",
        "tags": {
            "src_ip": src_ip,
            "dst_ip": dst_ip,
            "type": format!("{:?}", packet_type)
        },
        "fields": {
            "packets": flow.d_pkts,
            "bytes": flow.d_octets,
            "first_switched": flow.first,
            "last_switched": flow.last
        },
        "time": Utc::now()
    })
}
