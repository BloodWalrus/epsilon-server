// this will be a static skeleton for my prototype and i will probably use it for the main nea code as well

use std::{
    cell::Cell,
    fmt::Debug,
    iter::once,
    ops::{Index, IndexMut},
    rc::Rc,
};

use glam::{Quat, Vec3A};

use crate::{maths::calc_bone_base_from_bone_and_head, tree::BinaryTree};

const BONE_COUNT: usize = 9;
const JOINT_COUNT: usize = 8;

// Skeleton

#[derive(Debug)]
pub enum SkeletonError {
    InvalidStruture,
}

#[derive(Debug)]
pub struct Skeleton {
    tree: BinaryTree<(Bone, Option<Joint>)>,
    bones: Bones,
    joints: Joints,
    pub head_joint: Joint,
}

impl Skeleton {
    pub fn new(
        tree: BinaryTree<(Bone, Option<Joint>)>,
        bones: Bones,
        joints: Joints,
        head_joint: Joint,
    ) -> Self {
        Self {
            tree,
            bones,
            joints,
            head_joint,
        }
    }

    pub fn evaluate(&mut self) -> Result<(), SkeletonError> {
        Skeleton::evaluate_inner(&mut self.tree, &self.head_joint)
    }

    fn evaluate_inner(
        tree: &mut BinaryTree<(Bone, Option<Joint>)>,
        bone_head_joint: &Joint,
    ) -> Result<(), SkeletonError> {
        match tree.value {
            (ref bone, Some(ref joint)) => {
                let mut joint = joint.clone(); // making a new reference to the underlying joint data to avoid the borrow checker

                calc_bone_base_from_bone_and_head(bone_head_joint, bone, &mut joint);

                if let Some(tree) = tree.left_mut() {
                    Skeleton::evaluate_inner(tree, &mut joint)?
                }

                if let Some(tree) = tree.right_mut() {
                    Skeleton::evaluate_inner(tree, &mut joint)?
                }

                Ok(())
            }
            // if the tree node has to base joint then it should have no other tree nodes attatched
            _ => match (tree.left(), tree.right()) {
                (_, Some(_)) | (Some(_), _) => Err(SkeletonError::InvalidStruture),
                _ => Ok(()),
            },
        }
    }
}

impl Index<BoneId> for Skeleton {
    type Output = Bone;

    fn index(&self, index: BoneId) -> &Self::Output {
        &self.bones[index]
    }
}

impl IndexMut<BoneId> for Skeleton {
    fn index_mut(&mut self, index: BoneId) -> &mut Self::Output {
        &mut self.bones[index]
    }
}

impl Index<JointId> for Skeleton {
    type Output = Joint;

    fn index(&self, index: JointId) -> &Self::Output {
        &self.joints[index]
    }
}

impl IndexMut<JointId> for Skeleton {
    fn index_mut(&mut self, index: JointId) -> &mut Self::Output {
        &mut self.joints[index]
    }
}

impl Default for Skeleton {
    fn default() -> Self {
        let bones = Bones::default();
        let joints = Joints::default();
        let head_joint = joints[JointId::Head].clone();

        // setup default skeleton
        let left_foot = BinaryTree::new((bones[BoneId::LeftFoot].clone(), None), None, None);
        let right_foot = BinaryTree::new((bones[BoneId::RightFoot].clone(), None), None, None);

        let left_lower_leg = BinaryTree::new(
            (
                bones[BoneId::LeftLowerLeg].clone(),
                Some(joints[JointId::LeftAnkle].clone()),
            ),
            Some(left_foot),
            None,
        );
        let right_lower_leg = BinaryTree::new(
            (
                bones[BoneId::RightLowerLeg].clone(),
                Some(joints[JointId::RightAnkle].clone()),
            ),
            None,
            Some(right_foot),
        );

        let left_upper_leg = BinaryTree::new(
            (
                bones[BoneId::LeftUpperLeg].clone(),
                Some(joints[JointId::LeftKnee].clone()),
            ),
            Some(left_lower_leg),
            None,
        );
        let right_upper_leg = BinaryTree::new(
            (
                bones[BoneId::RightUpperLeg].clone(),
                Some(joints[JointId::RightKnee].clone()),
            ),
            None,
            Some(right_lower_leg),
        );

        let left_hip_offset = BinaryTree::new(
            (
                bones[BoneId::LeftHipOffset].clone(),
                Some(joints[JointId::LeftHipJoint].clone()),
            ),
            Some(left_upper_leg),
            None,
        );
        let right_hip_offset = BinaryTree::new(
            (
                bones[BoneId::RightHipOffset].clone(),
                Some(joints[JointId::RightHipJoint].clone()),
            ),
            None,
            Some(right_upper_leg),
        );

        let tree = BinaryTree::new(
            (
                bones[BoneId::Spine].clone(),
                Some(joints[JointId::Hips].clone()),
            ),
            Some(left_hip_offset),
            Some(right_hip_offset),
        );

        Self::new(tree, bones, joints, head_joint)
    }
}

// Joints

#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum JointId {
    Head,
    Hips,
    LeftHipJoint,
    LeftKnee,
    LeftAnkle,
    RightHipJoint,
    RightKnee,
    RightAnkle,
}

#[derive(Debug, Clone)]
pub struct JointData {
    pub position: Cell<Vec3A>,
}

#[derive(Debug, Clone)]
pub struct Joint(Rc<JointData>);

impl Joint {
    pub fn new(position: Vec3A) -> Self {
        Self(Rc::new(JointData {
            position: Cell::new(position),
        }))
    }

    pub fn get_position(&self) -> Vec3A {
        self.0.position.get()
    }

    pub fn set_position(&mut self, position: Vec3A) {
        self.0.position.set(position)
    }
}

impl Default for Joint {
    fn default() -> Self {
        Self::new(Vec3A::ZERO)
    }
}

pub struct Joints(Vec<Joint>);

impl Debug for Joints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[")?;
        for (i, joint) in self.0.iter().enumerate() {
            writeln!(
                f,
                "\t{:#?}: {},",
                unsafe { std::mem::transmute::<usize, JointId>(i) },
                joint.get_position()
            )?;
        }
        writeln!(f, "]")?;
        Ok(())
    }
}

impl Joints {
    pub fn as_mut_slice(&mut self) -> &mut [Joint] {
        self.0.as_mut_slice()
    }
}

impl Default for Joints {
    fn default() -> Self {
        Self(
            once(())
                .cycle()
                .take(JOINT_COUNT)
                .map(|_| Default::default())
                .collect(),
        )
    }
}

impl Index<JointId> for Joints {
    type Output = Joint;

    fn index(&self, index: JointId) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<JointId> for Joints {
    fn index_mut(&mut self, index: JointId) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

// Bones

#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum BoneId {
    Spine,
    LeftHipOffset,
    LeftUpperLeg,
    LeftLowerLeg,
    LeftFoot,
    RightHipOffset,
    RightUpperLeg,
    RightLowerLeg,
    RightFoot,
}

#[derive(Debug, Clone)]
pub struct BoneData {
    pub length: Cell<f32>,
    pub rotation: Cell<Quat>,
}

#[derive(Debug, Clone)]
pub struct Bone(Rc<BoneData>);

impl Bone {
    pub fn new(length: f32, rotation: Quat) -> Self {
        Self(Rc::new(BoneData {
            length: Cell::new(length),
            rotation: Cell::new(rotation),
        }))
    }

    pub fn get_length(&self) -> f32 {
        self.0.length.get()
    }

    pub fn get_rotation(&self) -> Quat {
        self.0.rotation.get()
    }

    pub fn set_length(&mut self, length: f32) {
        self.0.length.set(length)
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.0.rotation.set(rotation)
    }
}

impl Default for Bone {
    fn default() -> Self {
        Self::new(0.0, Quat::IDENTITY)
    }
}

pub struct Bones(Vec<Bone>);

impl Debug for Bones {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[")?;
        for (i, bone) in self.0.iter().enumerate() {
            writeln!(
                f,
                "\t{:#?}: {} | {}m,",
                unsafe { std::mem::transmute::<usize, BoneId>(i) },
                bone.get_rotation(),
                bone.get_length(),
            )?;
        }
        writeln!(f, "]")?;
        Ok(())
    }
}

impl Default for Bones {
    fn default() -> Self {
        Self(
            once(())
                .cycle()
                .take(BONE_COUNT)
                .map(|_| Default::default())
                .collect(),
        )
    }
}

impl Index<BoneId> for Bones {
    type Output = Bone;

    fn index(&self, index: BoneId) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<BoneId> for Bones {
    fn index_mut(&mut self, index: BoneId) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}
