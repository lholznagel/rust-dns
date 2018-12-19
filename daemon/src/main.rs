mod config;

use failure::Error;
use log::{debug, info};
use mio::{Events, Ready, Poll, PollOpt, Token};
use mio::net::UdpSocket;
use std::time::Duration;
use std::net::SocketAddr;
use std::collections::HashMap;

use rdns_proto::{DNS, ResourceRecord};

const SERVER: Token = Token(0);

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
enum RequestState {
    Added,
    ReadyToSend,
    WaitingForExternalServer
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct Request {
    pub requester: SocketAddr,
    pub state: RequestState,
    pub dns: DNS
}

fn main() -> Result<(), Error> {
    let mut pending_requests: HashMap<u16, Request> = HashMap::with_capacity(16);
    let mut known_addresses: HashMap<String, Vec<ResourceRecord>> = HashMap::with_capacity(128);

    loggify::Loggify::init_with_level(log::Level::Debug)?;

    let config = config::Config::load(String::from("./daemon/config.sample.yml"))?;
    info!("Using the following servers: {:?}", config.servers);

    let server = UdpSocket::bind(&"0.0.0.0:1337".parse()?)?;

    let poll = Poll::new()?;
    poll.register(&server, SERVER, Ready::all(), PollOpt::edge())?;

    let mut buffer = [0; 512];
    let mut events = Events::with_capacity(32);

    loop {
        poll.poll(&mut events, Some(Duration::from_millis(100)))?;
        for event in events.iter() {
            match event.token() {
                SERVER => {
                    if event.readiness().is_readable() {
                        let (num_recv, addr) = server.recv_from(&mut buffer)?;
                        let dns = DNS::parse(buffer[..num_recv].to_vec())?;

                        if pending_requests.contains_key(&dns.id) {
                            debug!("DNS request exists");
                            if !dns.resource_records.is_empty() {
                                let request = pending_requests.get(&dns.id).unwrap();
                                known_addresses.insert(dns.questions[0].qname.to_string(), dns.resource_records.clone());

                                pending_requests.insert(dns.id, Request {
                                    dns,
                                    state: RequestState::ReadyToSend,
                                    requester: request.requester
                                });
                            }
                        } else {
                            if known_addresses.contains_key(&dns.questions[0].qname) {
                                debug!("Cache Hit");
                                let mut dns = dns;
                                let address = known_addresses.get(&dns.questions[0].qname).unwrap();
                                dns.resource_records = address.to_vec();
                                dns.ancount = address.len() as u16;

                                pending_requests.insert(dns.id, Request {
                                    requester: addr,
                                    state: RequestState::ReadyToSend,
                                    dns
                                });
                            } else {
                                debug!("Cache Miss");
                                debug!("Adding new DNS request");
                                pending_requests.insert(dns.id, Request {
                                    requester: addr,
                                    state: RequestState::Added,
                                    dns
                                });
                            }
                        }

                        buffer = [0; 256];
                    }

                    if event.readiness().is_writable() {
                        for (key, value) in pending_requests.clone() {
                            if value.state == RequestState::ReadyToSend {
                                debug!("Sending complete DNS");
                                server.send_to(&value.dns.build(), &value.requester)?;
                                pending_requests.remove(&key);
                            } else if value.state == RequestState::Added {
                                debug!("Requesting from external server");

                                for server_addr in config.servers.iter() {
                                    let mut server_addr = server_addr.clone();
                                    server_addr.push_str(":53");

                                    debug!("Contacting {:?}", server_addr);
                                    server.send_to(&value.dns.clone().build(), &server_addr.parse()?)?;
                                }

                                pending_requests.insert(key, Request {
                                    dns: value.dns,
                                    state: RequestState::WaitingForExternalServer,
                                    requester: value.requester,
                                });
                            }
                        }
                    }
                },
                _ => unreachable!()
            }
        }
    }
}
