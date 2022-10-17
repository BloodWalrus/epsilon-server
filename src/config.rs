use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Deserialize)]
pub struct Config {
    pub streamer_sockets: Vec<SocketAddr>,
    pub driver_sockets: Vec<SocketAddr>,
    pub auto_switch_streamer: bool, // if you don't want it to crash if the streamer diconnnects, it will try the next one (it may actually be the same streamer thou)
}
