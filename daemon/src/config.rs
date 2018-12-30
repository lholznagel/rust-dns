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
    pub socket_path: String,
}

impl Config {
    pub fn load(path: String) -> Result<Self, Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let docs = YamlLoader::load_from_str(&contents)?;
        let docs = &docs[0];

        let mut hosts = HashMap::new();

        let hosts_file = Config::get_hosts_file(docs)?;
        for (key, value) in hosts_file {
            hosts.insert(key, value);
        }

        let hosts_config = Config::get_hosts_config(docs)?;
        for (key, value) in hosts_config {
            hosts.insert(key, value);
        }

        let listen_address = Config::get_listen_addr(docs)?;
        let servers = Config::get_servers(docs)?;
        let socket_path = Config::get_socket_path(docs)?;

        Ok(Self {
            hosts,
            listen_address,
            servers,
            socket_path,
        })
    }

    fn get_hosts_file(docs: &Yaml) -> Result<HashMap<String, Vec<ResourceRecord>>, Error> {
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

    fn get_hosts_config(docs: &Yaml) -> Result<HashMap<String, Vec<ResourceRecord>>, Error> {
        let has_hosts = !docs["hosts"].is_null();
        let mut hosts = HashMap::new();

        if has_hosts {
            for line in docs["hosts"].as_vec().unwrap() {
                for (key, value) in line.as_hash().unwrap() {
                    let ip: IpAddr = key.as_str().unwrap().parse()?;

                    if ip.is_ipv4() {
                        let ip: Ipv4Addr = key.as_str().unwrap().parse()?;

                        hosts.insert(
                            value.clone().into_string().unwrap(),
                            vec![ResourceRecord {
                                name: value.clone().into_string().unwrap(),
                                rtype: QType::A,
                                rclass: QClass::IN,
                                ttl: u32::max_value(), // should be long enough ~136 years
                                rdlength: 4,
                                rdata: ip.octets().to_vec(),
                            }],
                        );
                    } else {
                        let ip: Ipv6Addr = key.as_str().unwrap().parse()?;
                        let ip_octets = ip.octets().to_vec();

                        hosts.insert(
                            value.clone().into_string().unwrap(),
                            vec![ResourceRecord {
                                name: value.clone().into_string().unwrap(),
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

    fn get_socket_path(docs: &Yaml) -> Result<String, Error> {
        if !docs["socket_path"].is_null() {
            Ok(docs["socket_path"]
                .clone()
                .into_string()
                .unwrap_or_default())
        } else {
            Ok(String::new())
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hosts: HashMap::new(),
            listen_address: "0.0.0.0:53".parse().unwrap(),
            servers: Vec::new(),
            socket_path: String::new(),
        }
    }
}
