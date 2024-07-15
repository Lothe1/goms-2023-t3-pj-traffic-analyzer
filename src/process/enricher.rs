use chrono::Utc;
use netflow_parser::NetflowPacketResult;
use netflow_parser::NetflowParser;
use serde_json::json;
use crate::db::cidr_lookup::CidrLookup;
use crate::db::ip_lookup::{is_private_ip, IPtype};
use std::net::IpAddr;


pub async fn enrich_packet(payload: Vec<u8>, cidr_lookup: CidrLookup) -> Vec<Vec<u8>> {
    let mut enriched_packets: Vec<Vec<u8>> = Vec::new();
    if let Some(NetflowPacketResult::V5(packet)) = NetflowParser::default().parse_bytes(&payload).first() {
        println!("Parsing {} flows", packet.flowsets.len());
        for flow in &packet.flowsets {
            let src_ip = flow.src_addr.to_string();
            let dst_ip = flow.dst_addr.to_string();
            let src_country = cidr_lookup.lookup_country(&src_ip).unwrap_or(&"Unknown".to_string()).clone();
            let dst_country = cidr_lookup.lookup_country(&dst_ip).unwrap_or(&"Unknown".to_string()).clone();
            let (src_asn, src_as_name) = cidr_lookup.lookup_as(&src_ip).unwrap_or(&(String::from("Unknown"), String::from("Unknown"))).clone();
            let (dst_asn, dst_as_name) = cidr_lookup.lookup_as(&dst_ip).unwrap_or(&(String::from("Unknown"), String::from("Unknown"))).clone();
            
            let packet_type = match (is_private_ip(&src_ip), is_private_ip(&dst_ip)) {
                (true, true) => Some(IPtype::Incoming),
                (true, _) => Some(IPtype::Outgoing),
                (_, true) => Some(IPtype::Incoming),
                (_, _) => Some(IPtype::Outgoing)
            };

            if packet_type.is_some() {
                let time = Utc::now();
                let enriched_data = json!({
                    "measurement": "netflow",
                    "tags": {
                        "src_ip": src_ip.clone(),
                        "dst_ip": dst_ip.clone(),
                        "src_country": src_country.clone(),
                        "dst_country": dst_country.clone(),
                        "src_asn": src_asn.clone(),
                        "src_as_name": src_as_name.clone(),
                        "dst_asn": dst_asn.clone(),
                        "dst_as_name": dst_as_name.clone(),
                        "type": format!("{:?}", packet_type.unwrap()).clone()
                    },
                    "fields": {
                        "packets": flow.d_pkts,
                        "bytes": flow.d_octets,
                        "first_switched": flow.first,
                        "last_switched": flow.last
                    },
                    "time": time
                });
                println!("{:?}", enriched_data);
                let buf = serde_json::to_vec(&enriched_data).unwrap();
                enriched_packets.push(buf);
            } else {
                continue;
            }
        }
    }
    enriched_packets
}