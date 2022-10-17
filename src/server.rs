use crate::config::Config;
use crate::skeleton::*;
use crate::QUAT_ARRAY_SIZE;
use crate::SENSOR_COUNT;
use ecore::connection::{Client, Streamer};
use ecore::EpsilonResult;
use glam::Quat;
use glam::Vec3;
use std::mem::size_of;
use std::net::SocketAddr;

const JOINTS: [JointId; JOINT_COUNT] = [JointId::Hips, JointId::LeftAnkle, JointId::RightAnkle];
const JOINT_COUNT: usize = 3;
const JOINTS_SIZE: usize = size_of::<[JointId; JOINT_COUNT]>();
const BONES: [BoneId; BONE_COUNT] = [
    BoneId::Spine,
    BoneId::LeftHipOffset,
    BoneId::LeftUpperLeg,
    BoneId::LeftLowerLeg,
    BoneId::LeftFoot,
    BoneId::RightHipOffset,
    BoneId::RightUpperLeg,
    BoneId::RightLowerLeg,
    BoneId::RightFoot,
];
const BONE_COUNT: usize = 9;
const CONFIG_PATH: &str = "config.toml";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct JointPose {
    position: Vec3,
    rotation: Quat,
}

pub struct Server {
    streamer_client: Client<[Quat; SENSOR_COUNT], QUAT_ARRAY_SIZE>,
    driver_streamer: Streamer<[JointPose; JOINT_COUNT], JOINTS_SIZE>,
    streamer_clients: Option<Vec<SocketAddr>>,
    first_time_conn: bool,
    skeleton: Skeleton,
}

impl Server {
    pub fn new() -> EpsilonResult<Self> {
        let config_data = std::fs::read(CONFIG_PATH)?;
        let config: Config = toml::from_slice(&config_data)?;
        let streamer_client = Client::connect(&config.streamer_sockets[..])?;
        let driver_streamer = Streamer::listen(&config.driver_sockets[..])?;
        Ok(Self {
            streamer_client,
            driver_streamer,
            streamer_clients: if config.auto_switch_streamer {
                Some(config.streamer_sockets)
            } else {
                None
            },
            first_time_conn: true,
            skeleton: Default::default(),
        })
    }

    pub fn main(mut self) -> EpsilonResult<()> {
        'driver: loop {
            if let Err(err) = self.driver_streamer.next_client() {
                eprintln!("{:?}", err);
                continue 'driver;
            }
            'streamer: loop {
                if let Some(ref sockets) = self.streamer_clients {
                    if !self.first_time_conn {
                        self.streamer_client = Client::connect(&sockets[..])?;
                    }
                }

                loop {
                    // receive data from streamer
                    let tmp = if let Ok(tmp) = self.streamer_client.recv() {
                        tmp
                    } else {
                        self.first_time_conn = false;
                        continue 'streamer;
                    };

                    // map bone data to skeleton
                    for i in 0..BONE_COUNT {
                        self.skeleton[BONES[i]].set_rotation(tmp[i]);
                    }

                    // evaluate skeleton
                    self.skeleton.evaluate()?;

                    // extract joint data from skeleton into data structure that can be sent over the network
                    let _tmp = JOINTS.map(|joint| {
                        let joint = &self.skeleton[joint];
                        JointPose {
                            position: Vec3::from(joint.get_position()),
                            rotation: joint.get_rotation(),
                        }
                    });

                    // send over network
                    self.driver_streamer.send(_tmp)?;

                    if let Err(err) = self.driver_streamer.send(_tmp) {
                        eprintln!("{:?}", err);
                        self.first_time_conn = false;
                        continue 'streamer;
                    } // note to me
                      // some multithreading to have it source and sink streams running at different frequencys
                      // frame culling to reduce some "latency creep" if it occurs in testing
                      // should not be an issue if they are running localy thou
                }
            }
        }
    }
}
