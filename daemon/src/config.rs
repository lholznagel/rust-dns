use failure::{format_err, Error};
use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use yaml_rust::{yaml::Yaml, YamlLoader};

#[derive(Clone, Debug)]
pub struct Config {
    pub listen_address: SocketAddr,
    pub servers: Vec<String>,
}

impl Config {
    pub fn load(path: String) -> Result<Self, Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let docs = YamlLoader::load_from_str(&contents)?;
        let docs = &docs[0];

        let listen_address = Config::get_listen_addr(docs)?;
        let servers = Config::get_servers(docs)?;

        Ok(Self {
            listen_address,
            servers,
        })
    }

    fn get_listen_addr(docs: &Yaml) -> Result<SocketAddr, Error> {
        let addr = docs["listen-address"]
            .as_str()
            .unwrap_or("0.0.0.0:53")
            .to_string();

        if addr.contains(':') {
            Ok(addr.parse()?)
        } else {
            let ip = addr.parse()?;
            Ok(SocketAddr::new(ip, 53))
        }
    }

    fn get_servers(docs: &Yaml) -> Result<Vec<String>, Error> {
        let mut servers: Vec<String> = Vec::new();
        for doc in docs["servers"].clone() {
            let socket = match doc.as_str() {
                Some(v) => Ok(v),
                None => Err(format_err!("At least one server must be set.")),
            }?
            .to_string();
            servers.push(socket);
        }
        Ok(servers)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_address: "0.0.0.0:53".parse().unwrap(),
            servers: Vec::new(),
        }
    }
}
