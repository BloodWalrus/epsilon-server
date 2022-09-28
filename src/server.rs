use std::error::Error;

use crate::SENSOR_COUNT;
use ecore::{
    connection::Connection,
    skeleton::{JointId, Skeleton},
};
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
        loop {
            let tmp = self.streamer.recv()?;

            // map to skeleton

            self.skeleton.evaluate();

            let hips = &self.skeleton[JointId::Hips];
        }
    }
}
