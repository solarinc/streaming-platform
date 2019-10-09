use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::net::SocketAddr;
use serde_derive::Deserialize;
use warp::Filter;

#[derive(Debug, Deserialize)]
struct Config {
    host: String,
    cert_path: String,
    key_path: String
}

pub fn main() {    
    let config_path = std::env::args().nth(1)
        .expect("config location not passed as argument");

    let file = File::open(config_path)
        .expect("failed to open config");

    let mut buf_reader = BufReader::new(file);

    let mut config = String::new();

    buf_reader.read_to_string(&mut config)
        .expect("failed to read config");

    let config: Config = toml::from_str(&config)
        .expect("failed to deserialize config");

	let host = config.host.parse::<SocketAddr>().unwrap();

    let routes = warp::path("tmp")
    .and(warp::fs::dir("/tmp2"));

    warp::serve(routes)
        .tls(config.cert_path, config.key_path)
        .run(host);    
}
