#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{vec4, Mat3, Mat4, Vec2, Vec3, Vec4};
use spirv_std::{spirv, Image, Sampler};

#[spirv(vertex)]
pub fn vertex(
	position: Vec3,
	uv: Vec2,
	color: Vec3,
	normal: Vec3,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] mvp_mat: &Mat4,
	#[spirv(uniform, descriptor_set = 1, binding = 0)] normal_mat: &Mat3,
	// #[spirv(descriptor_set = 0, binding = 2)] light_dir: &Vec3,
	#[spirv(position)] clip_pos: &mut Vec4,
	out_uv: &mut Vec2,
	out_color: &mut Vec3,
	out_norm: &mut Vec3,
) {
	*out_uv = uv;
	*out_color = color;
	*out_norm = *normal_mat * normal;
	*clip_pos = *mvp_mat * position.extend(1.0);
}

#[spirv(fragment)]
pub fn fragment(
	in_uv: Vec2,
	in_color: Vec3,
	in_norm: Vec3,
	#[spirv(descriptor_set = 2, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 2, binding = 1)] sampler: &Sampler,
	frag_color: &mut Vec4,
) {
	let col = tex.sample(*sampler, in_uv);
	*frag_color = in_color.extend(1.0) * in_norm.extend(1.0).abs() * vec4(col.x, col.y, col.z, 1.0);
}
