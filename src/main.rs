use std::{error::Error, mem::size_of};

use glam::Quat;
use server::Server;

mod config;
mod maths;
mod server;
mod skeleton;
mod tree;

pub const SENSOR_COUNT: usize = 7;
pub const QUAT_ARRAY_SIZE: usize = size_of::<[Quat; SENSOR_COUNT]>();

fn main() -> Result<(), Box<dyn Error>> {
    Server::new()?.main()?;

    Ok(())
}
