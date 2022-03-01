use std::net::ToSocketAddrs;

use url::Url;

pub fn to_url(addr: String, port: u16) -> Url {
    let forward_socket_addr = (addr, port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .expect("given forwarding address was not valid");

    let forward_url = format!("http://{}", forward_socket_addr);
    Url::parse(&forward_url).unwrap()
}