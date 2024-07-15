// Returns whether a packet is Incoming or Outgoing
// i.e if the src IP = private -> outgoing
// else if dst IP = private -> outgoing

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum IPtype {
    Incoming,
    Outgoing,
}

pub fn is_private_ip(ip: &String) ->  bool {
    let ip_parts: Vec<u8> = ip.split('.').into_iter().map(|c| c.parse::<u8>().unwrap()).collect();
    // println!("{:?}", ip_parts);
    match ip_parts.get(0) {
        Some(&ip0) if ip0 == 10 => {
            // Any IP starting with 10 is private
            true
        }
        Some(&ip0) if ip0 == 127 => {
            match ip_parts.get(1) {
                Some(&ip1) if (ip1 >= 16 && ip1 <= 31) => true,
                Some(_) => false,
                None => false
            }
        }
        Some(&ip0) if ip0 == 192 => {
            match ip_parts.get(1) {
                Some(&ip1) if (ip1 == 168) => true,
                Some(_) => false,
                None => false
            }        
        }
        Some(_) => false,
        None => false
    }
}