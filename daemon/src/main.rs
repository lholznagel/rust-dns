mod config;
mod server;

use crate::server::ServerHandler;

use failure::Error;
use log::debug;
use mio::net::UdpSocket;
use mio::{Events, Poll, PollOpt, Ready, Token};
use std::time::Duration;

use rdns_proto::DNS;

const SERVER: Token = Token(0);

fn main() -> Result<(), Error> {
    let mut server_handler = ServerHandler::new();

    loggify::Loggify::init_with_level(log::Level::Debug)?;

    let config = config::Config::load(String::from("./daemon/config.sample.yml"))?;
    debug!("Config: {:?}", config);

    let server = UdpSocket::bind(&config.listen_address)?;

    let poll = Poll::new()?;
    poll.register(&server, SERVER, Ready::all(), PollOpt::edge())?;

    let mut buffer = [0; 512];
    let mut events = Events::with_capacity(32);

    loop {
        poll.poll(&mut events, Some(Duration::from_millis(100)))?;
        for event in events.iter() {
            match event.token() {
                SERVER => {
                    server_handler.validate_ttl()?;

                    if event.readiness().is_readable() {
                        let (num_recv, addr) = server.recv_from(&mut buffer)?;
                        let dns = DNS::parse(buffer[..num_recv].to_vec())?;

                        server_handler.read(addr, dns)?;
                    }

                    if event.readiness().is_writable() {
                        let (response, addrs) = server_handler.write(config.servers.clone())?;

                        for addr in addrs {
                            server.send_to(&response, &addr)?;
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
