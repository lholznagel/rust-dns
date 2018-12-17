mod config;

use failure::Error;
use log::{debug, info};
use mio::{Events, Ready, Poll, PollOpt, Token};
use mio::net::UdpSocket;
use std::time::Duration;
use std::net::SocketAddr;
use std::collections::HashMap;

use rdns_proto::DNS;

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
    let mut open_requests: HashMap<u16, Request> = HashMap::with_capacity(64);

    loggify::Loggify::init_with_level(log::Level::Trace)?;

    let config = config::Config::load(String::from("./daemon/config.sample.yml"))?;
    info!("Using the following servers: {:?}", config.servers);

    let server = UdpSocket::bind(&"0.0.0.0:1337".parse()?)?;

    let poll = Poll::new()?;
    poll.register(&server, SERVER, Ready::all(), PollOpt::edge())?;

    let mut buffer = [0; 256];
    let mut events = Events::with_capacity(128);

    loop {
        poll.poll(&mut events, Some(Duration::from_millis(100)))?;
        for event in events.iter() {
            match event.token() {
                SERVER => {
                    if event.readiness().is_readable() {
                        let (num_recv, addr) = server.recv_from(&mut buffer)?;
                        let dns = DNS::new(buffer[..num_recv].to_vec())?;

                        if open_requests.contains_key(&dns.id) {
                            debug!("DNS request exists");
                            if !dns.resource_records.is_empty() {
                                let request = open_requests.get(&dns.id).unwrap();

                                open_requests.insert(dns.id, Request {
                                    dns,
                                    state: RequestState::ReadyToSend,
                                    requester: request.requester
                                });
                            }
                        } else {
                            debug!("Adding new DNS request");
                            open_requests.insert(dns.id, Request {
                                requester: addr,
                                state: RequestState::Added,
                                dns
                            });
                        }

                        buffer = [0; 256];
                    }

                    if event.readiness().is_writable() {
                        for (key, value) in open_requests.clone() {
                            if value.state == RequestState::ReadyToSend {
                                debug!("Sending complete DNS");
                                server.send_to(&value.dns.build(), &value.requester)?;
                                open_requests.remove(&key);
                            } else if value.state == RequestState::Added {
                                debug!("Requesting from external server");
                                server.send_to(&value.dns.clone().build(), &"8.8.8.8:53".parse()?)?;

                                open_requests.insert(key, Request {
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
