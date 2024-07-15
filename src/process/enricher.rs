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

            println!("{:?}", packet_type);
            if packet_type.is_some() {
                let time = Utc::now();
                let enriched_data = json!({
                    "measurement": "netflow",
                    "tags": {
                        "src_ip": src_ip,
                        "dst_ip": dst_ip,
                        "src_country": src_country,
                        "dst_country": dst_country,
                        "src_asn": src_asn,
                        "src_as_name": src_as_name,
                        "dst_asn": dst_asn,
                        "dst_as_name": dst_as_name,
                        "type": format!("{:?}", packet_type.unwrap())
                    },
                    "fields": {
                        "packets": flow.d_pkts,
                        "bytes": flow.d_octets,
                        "first_switched": flow.first,
                        "last_switched": flow.last
                    },
                    "time": time
                });

                let buf = serde_json::to_vec(&enriched_data).unwrap();
                enriched_packets.push(buf);
            } else {
                continue;
            }
        }
    }
    enriched_packets
}

// pub async fn enrich_packet(payload: Vec<u8>, cidr_lookup: CidrLookup) -> Vec<Vec<u8>> {
//     let mut enriched_packets: Vec<Vec<u8>> = Vec::new();
//     if let NetflowPacketResult::V5(packet) = NetflowParser::default().parse_bytes(&payload).first().unwrap() {
//         println!("Begin for flow in packet.flowsets -- len {}", &packet.flowsets.len());
//         for flow in &packet.flowsets {

//             let src_ip = flow.src_addr.to_string();
//             let dst_ip = flow.dst_addr.to_string();
//             let src_country = cidr_lookup.lookup_country(&src_ip).unwrap();
//             let dst_country = cidr_lookup.lookup_country(&dst_ip).unwrap();
//             let src_as = cidr_lookup.lookup_as(&src_ip).unwrap();
//             let dst_as = cidr_lookup.lookup_as(&dst_ip).unwrap();
//             // println!("src private? [{}] -- dst private? [{}]", is_private_ip(&src_ip), is_private_ip(&dst_ip));
//             let packet_type = match (is_private_ip(&src_ip), is_private_ip(&dst_ip)) {
//                 (true, true) | (true, _) => IPtype::Outgoing,
//                 (_, true) => IPtype::Incoming,
//                 (_, _) => IPtype::Outgoing
//             };
//             let time = Utc::now();
//             let enriched_data = json!({
//                 "measurement": "netflow",
//                 "tags": {
//                     "src_ip": src_ip.clone(),
//                     "dst_ip": dst_ip.clone(),
//                     "src_country": src_country.clone(),
//                     "dst_country": dst_country.clone(),
//                     "src_as": src_as.clone(),
//                     "dst_as": dst_as.clone(),
//                     "type": format!("{:?}", packet_type).clone()
//                 },
//                 "fields": {
//                     "packets": flow.d_pkts, // Number of packets
//                     "bytes": flow.d_octets, // Number of bytes
//                     "first_switched": flow.first, // Start time of the flow
//                     "last_switched": flow.last, // End time of the flow
//                 },
//                 "time": time // Time of the flow (we'll use the packet's timestamp)
//             });

//             // println!("{:?}", enriched_data);
//             let buf = serde_json::to_vec(&enriched_data).unwrap();
//             enriched_packets.push(buf);
//         }
//     }
//     enriched_packets
// }