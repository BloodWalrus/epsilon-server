use crate::config::Config;
use crate::skeleton::*;
use ecore::connection::{CtrlSignal, Listener};
use ecore::constants::*;
use ecore::EpsilonResult;
use glam::Quat;

const TRACKERS: [(JointId, BoneId); TRACKER_COUNT] = [
    (JointId::Hips, BoneId::Spine),
    (JointId::LeftAnkle, BoneId::LeftFoot),
    (JointId::RightAnkle, BoneId::RightFoot),
];
const TRACKER_COUNT: usize = 3;

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
    streamer: (Listener<[Quat; SENSOR_COUNT]>, Listener<CtrlSignal>),
    driver: (Listener<[TrackerPose; TRACKER_COUNT]>, Listener<CtrlSignal>),
    skeleton: Skeleton,
}

impl Server {
    pub fn new() -> EpsilonResult<Self> {
        let config_data = std::fs::read(CONFIG_PATH)?;
        let config: Config = toml::from_slice(&config_data)?;
        let streamer = (
            Listener::listen(config.streamer_data)?,
            Listener::listen(config.streamer_ctrl)?,
        );
        let driver = (
            Listener::listen(config.driver_data)?,
            Listener::listen(config.driver_ctrl)?,
        );
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
            streamer,
            driver,
            skeleton,
        })
    }

    pub fn main(mut self) -> EpsilonResult<()> {
        for (mut streamer_data, mut streamer_ctrl) in
            self.streamer.0.incomming().zip(self.streamer.1.incomming())
        {
            for (mut driver_data, mut driver_ctrl) in
                self.driver.0.incomming().zip(self.driver.1.incomming())
            {
                loop {
                    // check for control signals and pass them on if there are any
                    if let Some(signal) = driver_ctrl.try_recv()? {
                        streamer_ctrl.send(&signal)?;
                    }

                    let tmp = streamer_data.recv()?;

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

                    // send over network
                    driver_data.send(&_tmp)?;
                }
            }
        }

        Ok(())
    }
}
