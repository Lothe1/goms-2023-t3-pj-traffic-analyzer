use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{self, BufRead};
use std::net::Ipv4Addr;
use std::path::Path;

#[derive(Debug)]
pub struct CidrLookup {
    // country code to country name
    country_map: HashMap<String, String>,
    // AS number to AS name
    as_map: HashMap<String, String>,
}

impl CidrLookup {
    
}