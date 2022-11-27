use crate::config::Config;
use crate::skeleton::*;
use crate::QUAT_ARRAY_SIZE;
use crate::SENSOR_COUNT;
use ecore::connection::{Client, Streamer};
use ecore::EpsilonResult;
use glam::Quat;
use std::mem::size_of;
use std::net::SocketAddr;

const JOINTS: [JointId; JOINT_COUNT] = [JointId::Hips, JointId::LeftAnkle, JointId::RightAnkle];
const JOINT_COUNT: usize = 3;
const JOINTS_SIZE: usize = size_of::<[JointPose; JOINT_COUNT]>();

const BONES: [BoneId; BONE_COUNT] = [
    BoneId::Spine,
    BoneId::LeftUpperLeg,
    BoneId::LeftLowerLeg,
    BoneId::LeftFoot,
    BoneId::RightUpperLeg,
    BoneId::RightLowerLeg,
    BoneId::RightFoot,
    BoneId::LeftHipOffset,
    BoneId::RightHipOffset,
];
const BONE_COUNT: usize = 9;
const CONFIG_PATH: &str = "config.toml";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct JointPose {
    position: [f32; 3],
    rotation: [f32; 4],
}

pub struct Server {
    streamer_client: Client<[Quat; SENSOR_COUNT], QUAT_ARRAY_SIZE>,
    driver_streamer: Streamer<[JointPose; JOINT_COUNT], JOINTS_SIZE>,
    streamer_clients: Option<Vec<SocketAddr>>,
    skeleton: Skeleton,
}

impl Server {
    pub fn new() -> EpsilonResult<Self> {
        let config_data = std::fs::read(CONFIG_PATH)?;
        let config: Config = toml::from_slice(&config_data)?;
        let streamer_client = Client::connect(&config.streamer_sockets[..])?;
        let driver_streamer = Streamer::listen(&config.driver_sockets[..])?;
        let mut skeleton = Skeleton::default();

        skeleton[BoneId::Spine].set_length(0.77);
        skeleton[BoneId::LeftHipOffset].set_length(0.15);
        skeleton[BoneId::LeftUpperLeg].set_length(0.48);
        skeleton[BoneId::LeftLowerLeg].set_length(0.42);
        skeleton[BoneId::LeftFoot].set_length(0.20);
        skeleton[BoneId::RightHipOffset].set_length(0.15);
        skeleton[BoneId::RightUpperLeg].set_length(0.48);
        skeleton[BoneId::RightLowerLeg].set_length(0.42);
        skeleton[BoneId::RightFoot].set_length(0.20);
        skeleton[BoneId::LeftHipOffset].set_rotation(Quat::from_rotation_z(90.0f32.to_radians()));
        skeleton[BoneId::RightHipOffset].set_rotation(Quat::from_rotation_z(-90.0f32.to_radians()));

        Ok(Self {
            streamer_client,
            driver_streamer,
            streamer_clients: if config.auto_switch_streamer {
                Some(config.streamer_sockets)
            } else {
                None
            },
            skeleton,
        })
    }

    pub fn main(mut self) -> EpsilonResult<()> {
        'driver: loop {
            if let Err(err) = self.driver_streamer.next_client() {
                eprintln!("{:?}", err);
                continue 'driver;
            }
            'streamer: loop {
                loop {
                    // receive data from streamer
                    let tmp = if let Ok(tmp) = self.streamer_client.recv() {
                        tmp
                    } else {
                        continue 'streamer;
                    };

                    // map bone data to skeleton
                    for i in 0..SENSOR_COUNT {
                        self.skeleton[BONES[i]].set_rotation(tmp[i]);
                    }

                    // evaluate skeleton
                    self.skeleton.evaluate()?;

                    // extract joint data from skeleton into data structure that can be sent over the network
                    let mut _tmp = JOINTS.map(|joint| {
                        let joint = &self.skeleton[joint];
                        JointPose {
                            position: joint.get_position().to_array(),
                            rotation: joint.get_rotation().to_array(),
                        }
                    });

                    _tmp[0].rotation = tmp[0].to_array();
                    _tmp[1].rotation = tmp[3].to_array();
                    _tmp[2].rotation = tmp[6].to_array();

                    // send over network
                    self.driver_streamer.send(_tmp)?;

                    if let Err(err) = self.driver_streamer.send(_tmp) {
                        eprintln!("{:?}", err);
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
