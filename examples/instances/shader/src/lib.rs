#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{Mat4, Vec3, Vec4};
use spirv_std::spirv;

#[spirv(vertex)]
pub fn vertex(
	position: Vec3,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] vp_mat: &Mat4,
	#[spirv(uniform, descriptor_set = 1, binding = 0)] model_mat: &Mat4,
	#[spirv(position)] clip_pos: &mut Vec4,
) {
	*clip_pos = *vp_mat * *model_mat * position.extend(1.0);
}

#[spirv(fragment)]
pub fn fragment(
	#[spirv(uniform, descriptor_set = 2, binding = 0)] color: &Vec4,
	frag_color: &mut Vec4,
) {
	*frag_color = *color * 1.0;
}
