use std::error::Error;

use server::Server;

mod interface;
mod server;

pub const SENSOR_COUNT: usize = 7;

fn main() -> Result<(), Box<dyn Error>> {
    Server::new().main()?;

    Ok(())
}
