#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{vec4, Mat4, Vec3, Vec4};
use spirv_std::spirv;

#[spirv(vertex)]
pub fn vs_main(
	pos: Vec3,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] mvp: &Mat4,
	#[spirv(position)] out_pos: &mut Vec4,
) {
	*out_pos = *mvp * pos.extend(1.0);
}

#[spirv(fragment)]
pub fn fs_main(#[spirv(uniform, descriptor_set = 0, binding = 1)] color: &Vec3, out: &mut Vec4) {
	*out = vec4(color.x, color.y, color.z, 1.0);
}
