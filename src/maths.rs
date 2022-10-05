use glam::Vec3A;

use crate::skeleton::{Bone, Joint};

const BONE_NO_ROTATION: Vec3A = Vec3A::NEG_Z;

pub fn calc_bone_base_from_bone_and_head(head: &Joint, bone: &Bone, base: &mut Joint) {
    base.set_position(
        head.get_position() + (bone.get_rotation().mul_vec3a(BONE_NO_ROTATION) * bone.get_length()),
    );
}
