use failure::Error;
use log::debug;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::SystemTime;

use rdns_proto::{ResourceRecord, DNS};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum RequestState {
    Added,
    ReadyToSend,
    WaitingForExternalServer,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Request {
    pub requester: SocketAddr,
    pub state: RequestState,
    pub dns: DNS,
}

#[derive(Clone, Debug)]
pub struct ServerHandler {
    pub pending_requests: HashMap<u16, Request>,
    pub known_addresses: HashMap<String, Vec<ResourceRecord>>,
    pub last_checked: SystemTime,
}

impl ServerHandler {
    pub fn new(hosts: HashMap<String, Vec<ResourceRecord>>) -> Self {
        let mut instance = Self {
            pending_requests: HashMap::with_capacity(16),
            known_addresses: HashMap::with_capacity(128),
            last_checked: SystemTime::now(),
        };

        for (key, value) in hosts {
            instance.known_addresses.insert(key, value);
        }

        instance
    }

    pub fn validate_ttl(&mut self) -> Result<(), Error> {
        for (key, value) in self.known_addresses.clone() {
            let mut updated_resources = Vec::new();
            for resource in value {
                let ttl = resource
                    .ttl
                    .checked_sub(self.last_checked.elapsed()?.as_secs() as u32)
                    .unwrap_or(0);

                if ttl > 0 {
                    updated_resources.push(ResourceRecord {
                        ttl: resource.ttl - self.last_checked.elapsed()?.as_secs() as u32,
                        ..resource.clone()
                    });
                }
            }

            if updated_resources.is_empty() {
                self.known_addresses.remove(&key);
            } else {
                self.known_addresses
                    .insert(key.to_string(), updated_resources);
            }
        }
        self.last_checked = SystemTime::now();
        Ok(())
    }

    pub fn read(&mut self, addr: SocketAddr, dns: DNS) -> Result<(), Error> {
        if self.known_addresses.contains_key(&dns.questions[0].qname) {
            debug!("Cache hit");
            let mut dns = dns.clone();
            let address = &self.known_addresses[&dns.questions[0].qname];
            dns.resource_records = address.to_vec();
        }

        if dns.resource_records.is_empty() {
            debug!("Adding new request");
            self.pending_requests.insert(
                dns.id,
                Request {
                    dns: dns.clone(),
                    state: RequestState::Added,
                    requester: addr,
                },
            );
        } else {
            self.known_addresses.insert(
                dns.questions[0].qname.to_string(),
                dns.resource_records.clone(),
            );
        }

        self.pending_requests.entry(dns.id).and_modify(|e| {
            if !dns.resource_records.is_empty() {
                let dns = dns.clone();
                e.state = RequestState::ReadyToSend;
                e.dns.resource_records = dns.resource_records.to_vec();
            }
        });

        Ok(())
    }

    pub fn write(&mut self, servers: Vec<String>) -> Result<(Vec<u8>, Vec<SocketAddr>), Error> {
        let mut response = Vec::new();
        let mut response_addr: Vec<SocketAddr> = Vec::with_capacity(servers.len());

        for (key, value) in self.pending_requests.clone() {
            if value.state == RequestState::ReadyToSend {
                debug!("Answering query");
                response = value.dns.build();
                response_addr.push(value.requester);
                self.pending_requests.remove(&key);
            } else if value.state == RequestState::Added {
                debug!("Requesting from external server");

                for server_addr in servers.iter() {
                    let mut server_addr = server_addr.clone();
                    server_addr.push_str(":53");

                    response = value.dns.clone().build();
                    response_addr.push(server_addr.parse()?);
                }

                self.pending_requests.insert(
                    key,
                    Request {
                        dns: value.dns,
                        state: RequestState::WaitingForExternalServer,
                        requester: value.requester,
                    },
                );
            }
        }
        Ok((response, response_addr))
    }

    pub fn addresses(&self) -> Vec<String> {
        let mut addresses = Vec::with_capacity(self.known_addresses.len());

        for (key, _) in self.known_addresses.clone() {
            addresses.push(key);
        }
        addresses
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rdns_proto::{QClass, QType, Question};

    #[test]
    pub fn test_read_query() {
        let mut server_handler = ServerHandler::new(HashMap::new());
        let dns = DNS {
            id: 13470,
            qr: 0,
            opcode: 0,
            aa: 0,
            tc: 0,
            rd: 1,
            ra: 0,
            z: 0,
            rcode: 0,
            qdcount: 1,
            ancount: 0,
            nscount: 0,
            arcount: 0,
            questions: vec![Question {
                qname: String::from("www.google.de"),
                qtype: QType::A,
                qclass: QClass::IN,
            }],
            resource_records: Vec::new(),
        };

        server_handler
            .read("0.0.0.0:1337".parse().unwrap(), dns)
            .unwrap();

        assert!(server_handler.pending_requests.len() == 1);
        assert!(server_handler.known_addresses.is_empty());
    }

    #[test]
    pub fn test_read_response() {
        let mut server_handler = ServerHandler::new(HashMap::new());
        let dns = DNS {
            id: 13470,
            qr: 1,
            opcode: 0,
            aa: 0,
            tc: 0,
            rd: 1,
            ra: 1,
            z: 0,
            rcode: 0,
            qdcount: 1,
            ancount: 1,
            nscount: 0,
            arcount: 0,
            questions: vec![Question {
                qname: String::from("www.google.de"),
                qtype: QType::A,
                qclass: QClass::IN,
            }],
            resource_records: vec![ResourceRecord {
                name: String::from("www.google.de"),
                rtype: QType::A,
                rclass: QClass::IN,
                ttl: 238,
                rdlength: 4,
                rdata: vec![172, 217, 168, 195],
            }],
        };

        server_handler
            .read("0.0.0.0:1337".parse().unwrap(), dns)
            .unwrap();

        assert!(server_handler.pending_requests.is_empty());
        assert!(server_handler.known_addresses.len() == 1);
    }

    #[test]
    pub fn test_cache_invalidates() {
        use std::thread;
        use std::time::Duration;

        let mut server_handler = ServerHandler::new(HashMap::new());
        let dns = DNS {
            id: 13470,
            qr: 1,
            opcode: 0,
            aa: 0,
            tc: 0,
            rd: 1,
            ra: 1,
            z: 0,
            rcode: 0,
            qdcount: 1,
            ancount: 1,
            nscount: 0,
            arcount: 0,
            questions: vec![Question {
                qname: String::from("www.google.de"),
                qtype: QType::A,
                qclass: QClass::IN,
            }],
            resource_records: vec![ResourceRecord {
                name: String::from("www.google.de"),
                rtype: QType::A,
                rclass: QClass::IN,
                ttl: 1,
                rdlength: 4,
                rdata: vec![172, 217, 168, 195],
            }],
        };

        server_handler
            .read("0.0.0.0:1337".parse().unwrap(), dns)
            .unwrap();

        thread::sleep(Duration::from_secs(1));
        server_handler.validate_ttl().unwrap();

        assert!(server_handler.pending_requests.is_empty());
        assert!(server_handler.known_addresses.is_empty());
    }
}
