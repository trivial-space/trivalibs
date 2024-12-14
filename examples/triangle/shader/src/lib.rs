#![no_std]
#![allow(unexpected_cfgs)]

#[cfg(target_arch = "spirv")]
use spirv_std::glam::{vec4, Vec2, Vec3, Vec4};
use spirv_std::{glam::vec2, spirv, Image, Sampler};
#[cfg(not(target_arch = "spirv"))]
use trivalibs::glam::{vec4, Vec2, Vec3, Vec4};
#[cfg(not(target_arch = "spirv"))]
use trivalibs::macros::*;

#[cfg(not(target_arch = "spirv"))]
#[apply(gpu_data)]
pub struct Vertex {
	pub position: Vec3,
	pub color: Vec3,
	pub uv: Vec2,
}

#[spirv(vertex)]
pub fn vertex(
	position: Vec3,
	color: Vec3,
	uv: Vec2,
	#[spirv(position)] clip_pos: &mut Vec4,
	out_color: &mut Vec3,
	out_uv: &mut Vec2,
) {
	*out_color = color;
	*out_uv = uv;
	*clip_pos = position.extend(1.0);
}

#[spirv(fragment)]
pub fn fragment(
	in_color: Vec3,
	in_uv: Vec2,
	#[spirv(descriptor_set = 0, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
	frag_color: &mut Vec4,
) {
	let col = tex.sample(*sampler, vec2(in_uv.x, in_uv.y));
	*frag_color = vec4(col.x, col.y, col.z, 1.0) * in_color.extend(1.0);
}
