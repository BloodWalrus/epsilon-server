use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Deserialize)]
pub struct Config {
    pub streamer_data: SocketAddr,
    pub streamer_ctrl: SocketAddr,
    pub driver_data: SocketAddr,
    pub driver_ctrl: SocketAddr,
    pub auto_switch_streamer: bool, // if you don't want it to crash if the streamer diconnnects, it will try the next one (it may actually be the same streamer thou)
}
