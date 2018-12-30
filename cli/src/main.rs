use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

fn main() {
    let mut stream = UnixStream::connect("rdns.sock").unwrap();
    stream.write_all(b"addresses").unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    println!("{}", response);
}
