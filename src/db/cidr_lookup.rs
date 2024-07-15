#![allow(unused_imports)]
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{self, BufRead};
use std::net::Ipv4Addr;
use std::path::Path;
use cidr::Ipv4Cidr;

use clap::builder::Str;

#[derive(Debug, Clone)]
pub struct CidrLookup {
    // country code to country name
    country_map: HashMap<String, String>,
    // AS number to AS name
    as_map: HashMap<String, (String, String)>,
}

impl CidrLookup {
    pub fn new(country_file: &str, as_file: &str) -> Self {
        let country_map = Self::load_country_cidr_map(country_file);
        let as_map = Self::load_as_cidr_map(as_file);
        CidrLookup { country_map, as_map }
    }

    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    // Load a CIDR map for countries from a TSV file
    fn load_country_cidr_map(file: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Ok(lines) = Self::read_lines(file) {
            for line in lines {
                if let Ok(line) = line {
                    let parts: Vec<&str> = line.split('\t').collect(); // Use '\t' for tab-separated files
                    if parts.len() >= 3 {
                        let ip_range = format!("{}-{}", parts[0], parts[1]);
                        let value = parts[2].to_string();
                        map.insert(ip_range, value);
                    }
                }
            }
        }
        map
    }

    // Load a CIDR map for AS numbers from a TSV file
    fn load_as_cidr_map(file: &str) -> HashMap<String, (String, String)> {
        let mut map = HashMap::new();
        if let Ok(lines) = Self::read_lines(file) {
            for line in lines {
                if let Ok(line) = line {
                    let parts: Vec<&str> = line.split('\t').collect(); // Use '\t' for tab-separated files
                    if parts.len() >= 5 {
                        let ip_range = format!("{}-{}", parts[0], parts[1]);
                        let asn = parts[2].to_string();
                        let as_name = parts[4].to_string();
                        map.insert(ip_range, (asn, as_name));
                    }
                }
            }
        }
        map
    }

    // Lookup the country for an IP address
    pub fn lookup_country(&self, ip: &str) -> Option<&String> {
        self.lookup_country_map(&self.country_map, ip)
    }

    // Lookup the AS for an IP address
    pub fn lookup_as(&self, ip: &str) -> Option<&(String, String)> {
        self.lookup_as_map(&self.as_map, ip)
    }

    // Lookup a value in a country CIDR map
    fn lookup_country_map<'a>(&'a self, map: &'a HashMap<String, String>, ip: &str) -> Option<&String> {
        let ip_addr: Ipv4Addr = ip.parse().unwrap();
        for (ip_range, value) in map.iter() {
            let range_parts: Vec<&str> = ip_range.split('-').collect();
            if range_parts.len() == 2 {
                let start_ip: Ipv4Addr = range_parts[0].parse().unwrap();
                let end_ip: Ipv4Addr = range_parts[1].parse().unwrap();
                if ip_addr >= start_ip && ip_addr <= end_ip {
                    return Some(value);
                }
            }
        }
        None
    }

    // Lookup a value in an AS CIDR map
    fn lookup_as_map<'a>(&'a self, map: &'a HashMap<String, (String, String)>, ip: &str) -> Option<&(String, String)> {
        let ip_addr: Ipv4Addr = ip.parse().unwrap();
        for (ip_range, value) in map.iter() {
            let range_parts: Vec<&str> = ip_range.split('-').collect();
            if range_parts.len() == 2 {
                let start_ip: Ipv4Addr = range_parts[0].parse().unwrap();
                let end_ip: Ipv4Addr = range_parts[1].parse().unwrap();
                if ip_addr >= start_ip && ip_addr <= end_ip {
                    return Some(value);
                }
            }
        }
        None
    }
}