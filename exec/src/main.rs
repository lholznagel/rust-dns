mod config;

use failure::Error;
use std::net::UdpSocket;

use rdns_proto::DNS;

fn main() -> Result<(), Error> {
    let config = config::Config::load(String::from("./exec/config.sample.yml"))?;
    println!("{:?}", config);
    /*{
        let socket = UdpSocket::bind("127.0.0.1:1337")?;

        let mut buf = [0; 256];
        let (amt, _) = socket.recv_from(&mut buf)?;

        println!("{:?}", DNS::new(buf[..amt].to_vec()));
    }*/
    Ok(())
}
