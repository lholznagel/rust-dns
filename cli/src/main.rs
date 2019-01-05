use clap::{App, Arg, SubCommand};

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

fn main() {
    let matches = App::new("RDNS Cli")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Lars Holznagel <contact@lholznagel.info>")
        .subcommand(
            SubCommand::with_name("metrics")
                .about("Gets the metrics of the dns server")
                .arg(
                    Arg::with_name("SOCKET")
                        .short("s")
                        .short("socket")
                        .help("Sets the socket path")
                        .takes_value(true),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("metrics") {
        let socket_path = matches.value_of("SOCKET").unwrap_or("rdns.sock");

        let mut stream = UnixStream::connect(socket_path).unwrap();
        stream.write_all(b"metrics").unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        println!("{}", response);
    }
}
