#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{Mat3, Mat4, Vec3, Vec4};
use spirv_std::spirv;

#[spirv(vertex)]
pub fn vertex(
	position: Vec3,
	color: Vec3,
	normal: Vec3,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] mvp_mat: &Mat4,
	#[spirv(uniform, descriptor_set = 1, binding = 0)] normal_mat: &Mat3,
	#[spirv(position)] clip_pos: &mut Vec4,
	out_color: &mut Vec3,
	out_norm: &mut Vec3,
) {
	*out_color = color;
	*out_norm = *normal_mat * normal;
	*clip_pos = *mvp_mat * position.extend(1.0);
}

#[spirv(fragment)]
pub fn fragment(in_color: Vec3, in_norm: Vec3, frag_color: &mut Vec4) {
	*frag_color = in_color.extend(1.0) * in_norm.extend(1.0).abs();
}
