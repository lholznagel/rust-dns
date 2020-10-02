mod error;
mod server;

use crate::server::ServerHandler;

use async_std::net::UdpSocket;
use rdns_proto::DNS;
use std::collections::HashMap;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    loggify::Loggify::init_with_level(log::Level::Debug).unwrap();

    let mut server_handler = ServerHandler::new(HashMap::new());

    let socket = UdpSocket::bind("127.0.0.1:1337").await?;
    let mut buf = vec![0u8; 512];

    loop {
        let (num_recv, addr) = socket.recv_from(&mut buf).await?;
        let dns = DNS::parse(buf[..num_recv].to_vec()).map_err(|e| {
            dbg!(e);
            error::RdnsError::Todo
        })?;
        dbg!(&dns);
        server_handler.read(addr, dns)?;

        let (response, addrs) = server_handler.write(vec!["8.8.8.8".into(), "8.8.4.4".into()])?;
        for addr in addrs {
            socket.send_to(&response, &addr).await?;
        }
    }
}
