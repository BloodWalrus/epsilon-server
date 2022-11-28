use crate::config::Config;
use crate::skeleton::*;
use crate::QUAT_ARRAY_SIZE;
use ecore::connection::{Client, Streamer};
use ecore::constants::*;
use ecore::EpsilonResult;
use glam::Quat;
use std::mem::size_of;
use std::net::SocketAddr;

const TRACKERS: [(JointId, BoneId); TRACKER_COUNT] = [
    (JointId::Hips, BoneId::Spine),
    (JointId::LeftAnkle, BoneId::LeftFoot),
    (JointId::RightAnkle, BoneId::RightFoot),
];
const TRACKER_COUNT: usize = 3;
const TRACKER_SIZE: usize = size_of::<[TrackerPose; TRACKER_COUNT]>();

// const BONES: [BoneId; BONE_COUNT] = [
//     BoneId::Spine,
//     BoneId::LeftUpperLeg,
//     BoneId::LeftLowerLeg,
//     BoneId::LeftFoot,
//     BoneId::RightUpperLeg,
//     BoneId::RightLowerLeg,
//     BoneId::RightFoot,
//     BoneId::LeftHipOffset,
//     BoneId::RightHipOffset,
// ];

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct TrackerPose {
    position: [f32; 3],
    rotation: [f32; 4],
}

pub struct Server {
    streamer_client: Client<[Quat; SENSOR_COUNT], QUAT_ARRAY_SIZE>,
    driver_streamer: Streamer<[TrackerPose; TRACKER_COUNT], TRACKER_SIZE>,
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
                        self.skeleton[BoneId::from(i)].set_rotation(tmp[i]);
                    }

                    // evaluate skeleton
                    self.skeleton.evaluate()?;

                    // extract joint data from skeleton into data structure that can be sent over the network
                    let mut _tmp = TRACKERS.map(|(joint, bone)| {
                        let joint = &self.skeleton[joint];
                        TrackerPose {
                            position: joint.get_position().to_array(),
                            rotation: tmp[bone as usize].to_array(),
                        }
                    });

                    // _tmp[0].rotation = tmp[BoneId::Spine as usize].to_array();
                    // _tmp[1].rotation = tmp[BoneId::LeftFoot as usize].to_array();
                    // _tmp[2].rotation = tmp[BoneId::RightFoot as usize].to_array();

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
