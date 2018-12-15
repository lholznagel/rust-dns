use failure::{format_err, Error};
use std::fs::File;
use std::io::Read;
use yaml_rust::YamlLoader;

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub servers: Vec<String>,
}

impl Config {
    pub fn load(path: String) -> Result<Self, Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let docs = YamlLoader::load_from_str(&contents)?;
        let docs = &docs[0];

        let mut servers: Vec<String> = Vec::new();
        for doc in docs["servers"].clone() {
            let socket = match doc.as_str() {
                Some(v) => Ok(v),
                None => Err(format_err!("At least one server must be set.")),
            }?
            .to_string();
            servers.push(socket);
        }

        Ok(Self { servers })
    }
}
