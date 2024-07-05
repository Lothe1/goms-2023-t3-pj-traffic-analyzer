use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{self, BufRead};
use std::net::Ipv4Addr;
use std::path::Path;
use cidr::Ipv4Cidr;

use clap::builder::Str;

#[derive(Debug)]
pub struct CidrLookup {
    // country code to country name
    country_map: HashMap<String, String>,
    // AS number to AS name
    as_map: HashMap<String, String>,
}

impl CidrLookup {
    pub fn new(country_file: &str, as_file: &str) -> Self {
        let country_map = Self::load_cidr_map(country_file);
        let as_map = Self::load_cidr_map(as_file);
        CidrLookup { country_map, as_map }
    }

    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    // Load a CIDR map from a file
    // The file should be a CSV file with the CIDR block in the first column
    // and the value in the second column
    fn load_cidr_map(file: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Ok(lines) = Self::read_lines(file) {
            for line in lines {
                if let Ok(line) = line {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() == 2 {
                        map.insert(parts[0].to_string(), parts[1].to_string());
                    }
                }
            }
        }
        map
    }

    // Lookup the country for an IP address
    pub fn lookup_country(&self, ip: &str) -> Option<&String> {
        self.lookup(&self.country_map, ip)
    }

    // Lookup the AS for an IP address
    pub fn lookup_as(&self, ip: &str) -> Option<&String> {
        self.lookup(&self.as_map, ip)
    }

    // Lookup a value in a CIDR map
    fn lookup<'a>(&'a self, map: &'a HashMap<String, String>, ip: &str) -> Option<&String> {
        let ip_addr: Ipv4Addr = ip.parse().unwrap();
        for cidr in map.keys() {
            if cidr::Ipv4Cidr::new(ip_addr, cidr.parse().unwrap()).is_ok() {
                return map.get(cidr);
            }
        }
        None
    }
}