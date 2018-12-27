use rdns_proto::{QClass, QType, ResourceRecord};

use failure::{format_err, Error};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use yaml_rust::{yaml::Yaml, YamlLoader};

#[derive(Clone, Debug)]
pub struct Config {
    pub hosts: HashMap<String, Vec<ResourceRecord>>,
    pub listen_address: SocketAddr,
    pub servers: Vec<String>,
}

impl Config {
    pub fn load(path: String) -> Result<Self, Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let docs = YamlLoader::load_from_str(&contents)?;
        let docs = &docs[0];

        let hosts = Config::get_hosts(docs)?;
        let listen_address = Config::get_listen_addr(docs)?;
        let servers = Config::get_servers(docs)?;

        Ok(Self {
            hosts,
            listen_address,
            servers,
        })
    }

    fn get_hosts(docs: &Yaml) -> Result<HashMap<String, Vec<ResourceRecord>>, Error> {
        let addr = docs["load_hosts_file"].as_bool().unwrap_or(false);
        let mut hosts = HashMap::new();

        if addr {
            let f = BufReader::new(File::open("/etc/hosts")?);
            for line in f.lines() {
                let line = line?;

                if !line.starts_with('#') {
                    let splitted: Vec<&str> = line.split(' ').collect();

                    let ip: IpAddr = splitted[0].parse()?;

                    if ip.is_ipv4() {
                        let ip: Ipv4Addr = splitted[0].parse()?;

                        hosts.insert(
                            splitted.last().unwrap().to_string(),
                            vec![ResourceRecord {
                                name: splitted.last().unwrap().to_string(),
                                rtype: QType::A,
                                rclass: QClass::IN,
                                ttl: u32::max_value(), // should be long enough ~136 years
                                rdlength: 4,
                                rdata: ip.octets().to_vec(),
                            }],
                        );
                    } else {
                        let ip: Ipv6Addr = splitted[0].parse()?;
                        let ip_octets = ip.octets().to_vec();

                        hosts.insert(
                            splitted.last().unwrap().to_string(),
                            vec![ResourceRecord {
                                name: splitted.last().unwrap().to_string(),
                                rtype: QType::AAAA,
                                rclass: QClass::IN,
                                ttl: u32::max_value(), // should be long enough ~136 years
                                rdlength: ip_octets.len() as u16,
                                rdata: ip_octets,
                            }],
                        );
                    }
                }
            }
        }

        Ok(hosts)
    }

    fn get_listen_addr(docs: &Yaml) -> Result<SocketAddr, Error> {
        let addr = docs["listen-address"]
            .as_str()
            .unwrap_or("0.0.0.0:53")
            .to_string();

        if addr.contains(':') {
            Ok(addr.parse()?)
        } else {
            let ip = addr.parse()?;
            Ok(SocketAddr::new(ip, 53))
        }
    }

    fn get_servers(docs: &Yaml) -> Result<Vec<String>, Error> {
        let mut servers: Vec<String> = Vec::new();
        for doc in docs["servers"].clone() {
            let socket = match doc.as_str() {
                Some(v) => Ok(v),
                None => Err(format_err!("At least one server must be set.")),
            }?
            .to_string();
            servers.push(socket);
        }
        Ok(servers)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hosts: HashMap::new(),
            listen_address: "0.0.0.0:53".parse().unwrap(),
            servers: Vec::new(),
        }
    }
}
