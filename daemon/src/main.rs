mod config;
mod server;
mod stats;

use crate::server::ServerHandler;
use crate::stats::Stats;

use failure::Error;
use log::debug;
use mio::net::UdpSocket;
use mio::{Events, Poll, PollOpt, Ready, Token};
use mio_uds::UnixListener;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

use rdns_proto::DNS;

const SERVER: Token = Token(0);
const UNIX: Token = Token(1);

fn main() -> Result<(), Error> {
    loggify::Loggify::init_with_level(log::Level::Debug)?;

    let config = config::Config::load(String::from("./daemon/config.sample.yml"))?;
    let mut server_handler = ServerHandler::new(config.hosts.clone());
    debug!("Config: {:?}", config);

    remove_socket_if_exists(config.socket_path.clone());

    let server = UdpSocket::bind(&config.listen_address)?;
    let unix_socket = UnixListener::bind(&config.socket_path)?;

    let poll = Poll::new()?;
    poll.register(&server, SERVER, Ready::all(), PollOpt::edge())?;
    poll.register(&unix_socket, UNIX, Ready::readable(), PollOpt::edge())?;

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
                UNIX => {
                    let (mut stream, _) = unix_socket.accept()?.unwrap();
                    let mut result = Vec::new();
                    let _ = stream.read_to_end(&mut result);

                    let message = String::from_utf8(result)?;
                    if message == "addresses" {
                        stream.write_all(&server_handler.stats())?;
                    } else {
                        stream.write_all(b"Unknown command")?;
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

fn remove_socket_if_exists(path: String) {
    if Path::new(&path).exists() {
        fs::remove_file(path).unwrap();
    }
}
