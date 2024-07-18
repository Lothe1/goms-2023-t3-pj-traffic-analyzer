use chrono::Utc;
use netflow_parser::static_versions::v5::{FlowSet, Header};
use netflow_parser::variable_versions::common::FieldValue;
use netflow_parser::variable_versions::ipfix_lookup::IPFixField;
use netflow_parser::variable_versions::v9_lookup::V9Field;
use netflow_parser::{NetflowPacketResult, NetflowParser};
use netflow_parser::static_versions::v5::FlowSet as FlowSetV5;
use netflow_parser::variable_versions::v9::{FlowSet as FlowSetV9, TemplateField as TemplateFieldV9};
use netflow_parser::variable_versions::ipfix::{FlowSet as FlowSetIPFix, TemplateField as TemplateFieldIPFix};
use std::collections::HashMap;
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
                    // enrich_flow_v9(flow, &flow.body.templates, &cidr_lookup, &mut enriched_packets);
                    enrich_flow_v9(flow, &cidr_lookup, &mut enriched_packets);
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


// NetFlow v5
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


// NetFlow v9
fn enrich_flow_v9(flow: &netflow_parser::variable_versions::v9::FlowSet, 
                  cidr_lookup: &CidrLookup, 
                  enriched_packets: &mut Vec<Vec<u8>>) {

    if let Some(f) = &flow.body.data {
        for data_record in &f.data_fields {
            let mut src_ip = String::new();
            let mut dst_ip = String::new();
            let mut packets = 0;
            let mut bytes = 0;
            let mut first_switched = 0;
            let mut last_switched = 0;

            for (_, (_, (field_type, field_value))) in data_record.iter().enumerate() {
                match field_type {
                    V9Field::Ipv4SrcAddr => {
                        src_ip = extract_ip_address(&field_value).unwrap_or("Unknown".to_string());
                    },
                    V9Field::Ipv4DstAddr => {
                        dst_ip = extract_ip_address(&field_value).unwrap_or("Unknown".to_string());
                    },
                    V9Field::InPkts => {
                        packets = match field_value {
                            FieldValue::Float64(val) => *val as u64,
                            _ => 0
                        };
                    },
                    V9Field::InBytes => {
                        bytes = match field_value {
                            FieldValue::Float64(val) => *val as u32,
                            _ => 0
                        };
                    }, 
                    V9Field::FirstSwitched => {
                        first_switched = match field_value {
                            FieldValue::Float64(val) => *val as u64,
                            _ => 0
                        };
                    },
                    V9Field::LastSwitched => {
                        last_switched = match field_value {
                            FieldValue::Float64(val) => *val as u64,
                            _ => 0
                        };
                    },
                    _ => { }
                }
            }

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
                        "packets": packets,
                        "bytes": bytes,
                        "first_switched": first_switched,
                        "last_switched": last_switched
                    },
                    "time": time
                });
                println!("{:?}", enriched_data);
                let buf = serde_json::to_vec(&enriched_data).unwrap();
                enriched_packets.push(buf);
            }
        }
    }
}


// Extract IP address from field value
fn extract_ip_address(field_val: &FieldValue) -> Option<String> {
    match field_val {
        FieldValue::Ip4Addr(ip) => Some(ip.to_string()),
        _ => None
    }
}


// IPFIX
fn enrich_flow_ipfix(flow: &netflow_parser::variable_versions::ipfix::FlowSet, 
                     cidr_lookup: &CidrLookup, 
                     enriched_packets: &mut Vec<Vec<u8>>) {
    if let Some(f) = &flow.body.data {
        for data_record in &f.data_fields {
            let mut src_ip = String::new();
            let mut dst_ip = String::new();
            let mut packets = 0;
            let mut bytes = 0;
            let mut first_switched = 0;
            let mut last_switched = 0;

            for (_, (_, (field_type, field_value))) in data_record.iter().enumerate() {
                match field_type {
                    IPFixField::SourceIpv4address => {
                        src_ip = extract_ip_address(&field_value).unwrap_or("Unknown".to_string());
                    },
                    IPFixField::DestinationIpv4address => {
                        dst_ip = extract_ip_address(&field_value).unwrap_or("Unknown".to_string());
                    },
                    IPFixField::PacketDeltaCount => {
                        packets = match field_value {
                            FieldValue::Float64(val) => *val as u64,
                            _ => 0
                        };
                    },
                    IPFixField::OctetDeltaCount => {
                        bytes = match field_value {
                            FieldValue::Float64(val) => *val as u32,
                            _ => 0
                        };
                    }, 
                    IPFixField::FlowStartSeconds => {
                        first_switched = match field_value {
                            FieldValue::Float64(val) => *val as u64,
                            _ => 0
                        };
                    },
                    IPFixField::FlowEndSeconds => {
                        last_switched = match field_value {
                            FieldValue::Float64(val) => *val as u64,
                            _ => 0
                        };
                    },
                    _ => { }
                }
            }

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
                        "packets": packets,
                        "bytes": bytes,
                        "first_switched": first_switched,
                        "last_switched": last_switched
                    },
                    "time": time
                });
                println!("{:?}", enriched_data);
                let buf = serde_json::to_vec(&enriched_data).unwrap();
                enriched_packets.push(buf);
            }
        }
    }
}