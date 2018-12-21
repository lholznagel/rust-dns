mod config;

use failure::Error;
use log::{debug, info};
use mio::net::UdpSocket;
use mio::{Events, Poll, PollOpt, Ready, Token};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use rdns_proto::{ResourceRecord, DNS};

const SERVER: Token = Token(0);

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
enum RequestState {
    Added,
    ReadyToSend,
    WaitingForExternalServer,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct Request {
    pub requester: SocketAddr,
    pub state: RequestState,
    pub dns: DNS,
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

                        if known_addresses.contains_key(&dns.questions[0].qname) {
                            debug!("Cache Hit");
                            let mut dns = dns.clone();
                            let address = &known_addresses[&dns.questions[0].qname];
                            dns.resource_records = address.to_vec();

                            pending_requests.insert(
                                dns.id,
                                Request {
                                    dns,
                                    requester: addr,
                                    state: RequestState::ReadyToSend,
                                },
                            );
                        }

                        pending_requests
                            .entry(dns.id)
                            .and_modify(|e| {
                                let dns = dns.clone();
                                if !dns.resource_records.is_empty() {
                                    e.state = RequestState::ReadyToSend;
                                    e.dns.resource_records = dns.resource_records.to_vec();

                                    known_addresses.insert(
                                        dns.questions[0].qname.to_string(),
                                        dns.resource_records.clone(),
                                    );
                                }
                            })
                            .or_insert(Request {
                                dns,
                                state: RequestState::Added,
                                requester: addr,
                            });
                    }

                    if event.readiness().is_writable() {
                        println!("2");
                        for (key, value) in pending_requests.clone() {
                            println!("3 {:?}", value);
                            if value.state == RequestState::ReadyToSend {
                                debug!("Answering query");
                                server.send_to(&value.dns.build(), &value.requester)?;
                                pending_requests.remove(&key);
                            } else if value.state == RequestState::Added {
                                debug!("Requesting from external server");

                                for server_addr in config.servers.iter() {
                                    let mut server_addr = server_addr.clone();
                                    server_addr.push_str(":53");

                                    debug!("Contacting {:?}", server_addr);
                                    server.send_to(
                                        &value.dns.clone().build(),
                                        &server_addr.parse()?,
                                    )?;
                                }

                                pending_requests.insert(
                                    key,
                                    Request {
                                        dns: value.dns,
                                        state: RequestState::WaitingForExternalServer,
                                        requester: value.requester,
                                    },
                                );
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
