use chrono::Utc;
use netflow_parser::static_versions::v5::{FlowSet, Header};
use netflow_parser::{NetflowPacketResult, NetflowParser};
use serde_json::json;
use crate::db::cidr_lookup::CidrLookup;
use crate::db::ip_lookup::{is_private_ip, IPtype};
use std::net::IpAddr;


pub async fn enrich_packet(payload: Vec<u8>, cidr_lookup: CidrLookup) -> Vec<Vec<u8>> {
    let mut enriched_packets: Vec<Vec<u8>> = Vec::new();

    let mut parser = NetflowParser::default();
    for packet_result in parser.parse_bytes(&payload) {
        match packet_result {
            NetflowPacketResult::V5(packet) => {
                println!("Parsing NetFlow v5 with {} flows", packet.flowsets.len());
                for flow in &packet.flowsets {
                    enrich_flow_v5(flow, &cidr_lookup, &mut enriched_packets);
                }
            },
            NetflowPacketResult::V9(packet) => {
                println!("Parsing NetFlow v9 with {} flows", packet.flowsets.len());
                for flow in &packet.flowsets {
                    enrich_flow_v9(flow, &packet.header, &cidr_lookup, &mut enriched_packets);
                }
            },
            NetflowPacketResult::IPFix(packet) => {
                println!("Parsing IPFIX with {} flows", packet.flowsets.len());
                for flow in &packet.flowsets {
                    enrich_flow_ipfix(flow, &cidr_lookup, &mut enriched_packets);
                }
            },
            _ => {
                // Handle other versions or unsupported cases
                println!("Unsupported NetFlow version");
            }
        }
    }

    enriched_packets
}

fn enrich_flow_v5(flow: &netflow_parser::static_versions::v5::FlowSet, 
                  cidr_lookup: &CidrLookup, 
                  enriched_packets: &mut Vec<Vec<u8>>) {
    
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

    if let Some(packet_type) = packet_type {
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
                "type": format!("{:?}", packet_type).clone()
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
    }
}


fn enrich_flow_v9(flow: &netflow_parser::variable_versions::v9::FlowSet, 
                 header: &netflow_parser::variable_versions::v9::Header, 
                 cidr_lookup: &CidrLookup, 
                 enriched_packets: &mut Vec<Vec<u8>>) {

    // // Example: Accessing fields directly if supported by FlowSet in v9
    // let src_ip = match header.source_address {
    //     Some(netflow_parser::variable_versions::v9::Address::IPv4(addr)) => IpAddr::V4(addr),
    //     Some(netflow_parser::variable_versions::v9::Address::IPv6(addr)) => IpAddr::V6(addr),
    //     None => IpAddr::V4("0.0.0.0".parse().unwrap()), // Example default
    // };

    // let dst_ip = match header.destination_address {
    //     Some(netflow_parser::variable_versions::v9::Address::IPv4(addr)) => IpAddr::V4(addr),
    //     Some(netflow_parser::variable_versions::v9::Address::IPv6(addr)) => IpAddr::V6(addr),
    //     None => IpAddr::V4("0.0.0.0".parse().unwrap()), // Example default
    // };

    // let src_ip_str = src_ip.to_string();
    // let dst_ip_str = dst_ip.to_string();

    // let src_country = cidr_lookup.lookup_country(&src_ip_str).unwrap_or(&"Unknown".to_string()).clone();
    // let dst_country = cidr_lookup.lookup_country(&dst_ip_str).unwrap_or(&"Unknown".to_string()).clone();
    // let (src_asn, src_as_name) = cidr_lookup.lookup_as(&src_ip_str).unwrap_or(&(String::from("Unknown"), String::from("Unknown"))).clone();
    // let (dst_asn, dst_as_name) = cidr_lookup.lookup_as(&dst_ip_str).unwrap_or(&(String::from("Unknown"), String::from("Unknown"))).clone();

    // let packet_type = match (is_private_ip(&src_ip_str), is_private_ip(&dst_ip_str)) {
    //     (true, true) => Some(IPtype::Incoming),
    //     (true, _) => Some(IPtype::Outgoing),
    //     (_, true) => Some(IPtype::Incoming),
    //     (_, _) => Some(IPtype::Outgoing)
    // };

    // if let Some(packet_type) = packet_type {
    //     let time = Utc::now();
    //     let enriched_data = json!({
    //         "measurement": "netflow",
    //         "tags": {
    //         "src_ip": src_ip_str.clone(),
    //         "dst_ip": dst_ip_str.clone(),
    //         "src_country": src_country.clone(),
    //         "dst_country": dst_country.clone(),
    //         "src_asn": src_asn.clone(),
    //         "src_as_name": src_as_name.clone(),
    //         "dst_asn": dst_asn.clone(),
    //         "dst_as_name": dst_as_name.clone(),
    //         "type": format!("{:?}", packet_type).clone()
    //     },
    //     "fields": {
    //         "packets": flow.data_bytes, 
    //         "bytes": flow.data_packets, 
    //         "first_switched": flow.flow_start, 
    //         "last_switched": flow.flow_end
    //     },
    //     "time": time
    //     });
    //     println!("{:?}", enriched_data);
    //     let buf = serde_json::to_vec(&enriched_data).unwrap();
    //     enriched_packets.push(buf);
    // }
}




fn enrich_flow_ipfix(flow: &netflow_parser::variable_versions::ipfix::FlowSet, cidr_lookup: &CidrLookup, enriched_packets: &mut Vec<Vec<u8>>) {

}


// pub async fn enrich_packet(payload: Vec<u8>, cidr_lookup: CidrLookup) -> Vec<Vec<u8>> {
//     let mut enriched_packets: Vec<Vec<u8>> = Vec::new();
//     if let Some(NetflowPacketResult::V5(packet)) = NetflowParser::default().parse_bytes(&payload).first() {
//         println!("Parsing {} flows", packet.flowsets.len());
//         for flow in &packet.flowsets {
//             let src_ip = flow.src_addr.to_string();
//             let dst_ip = flow.dst_addr.to_string();
//             let src_country = cidr_lookup.lookup_country(&src_ip).unwrap_or(&"Unknown".to_string()).clone();
//             let dst_country = cidr_lookup.lookup_country(&dst_ip).unwrap_or(&"Unknown".to_string()).clone();
//             let (src_asn, src_as_name) = cidr_lookup.lookup_as(&src_ip).unwrap_or(&(String::from("Unknown"), String::from("Unknown"))).clone();
//             let (dst_asn, dst_as_name) = cidr_lookup.lookup_as(&dst_ip).unwrap_or(&(String::from("Unknown"), String::from("Unknown"))).clone();
            
//             let packet_type = match (is_private_ip(&src_ip), is_private_ip(&dst_ip)) {
//                 (true, true) => Some(IPtype::Incoming),
//                 (true, _) => Some(IPtype::Outgoing),
//                 (_, true) => Some(IPtype::Incoming),
//                 (_, _) => Some(IPtype::Outgoing)
//             };

//             if packet_type.is_some() {
//                 let time = Utc::now();
//                 let enriched_data = json!({
//                     "measurement": "netflow",
//                     "tags": {
//                         "src_ip": src_ip.clone(),
//                         "dst_ip": dst_ip.clone(),
//                         "src_country": src_country.clone(),
//                         "dst_country": dst_country.clone(),
//                         "src_asn": src_asn.clone(),
//                         "src_as_name": src_as_name.clone(),
//                         "dst_asn": dst_asn.clone(),
//                         "dst_as_name": dst_as_name.clone(),
//                         "type": format!("{:?}", packet_type.unwrap()).clone()
//                     },
//                     "fields": {
//                         "packets": flow.d_pkts,
//                         "bytes": flow.d_octets,
//                         "first_switched": flow.first,
//                         "last_switched": flow.last
//                     },
//                     "time": time
//                 });
//                 println!("{:?}", enriched_data);
//                 let buf = serde_json::to_vec(&enriched_data).unwrap();
//                 enriched_packets.push(buf);
//             } else {
//                 continue;
//             }
//         }
//     }
//     enriched_packets
// }