use std::error::Error;

use crate::SENSOR_COUNT;
use ecore::{connection::Connection, skeleton::Skeleton};
use glam::Quat;

pub struct Server {
    streamer: Connection<SENSOR_COUNT, Quat>,
    skeleton: Skeleton,
}

impl Server {
    pub fn new() -> Self {
        Self {
            streamer: todo!(),
            skeleton: Default::default(),
        }
    }

    pub fn main(mut self) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
