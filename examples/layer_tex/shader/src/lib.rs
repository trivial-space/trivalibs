#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{vec4, Mat4, Vec2, Vec3, Vec4};
use spirv_std::{spirv, Image, Sampler};

#[spirv(vertex)]
pub fn color_vs(
	pos: Vec3,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] mvp: &Mat4,
	#[spirv(position)] out_pos: &mut Vec4,
) {
	*out_pos = *mvp * pos.extend(1.0);
}

#[spirv(fragment)]
pub fn color_fs(#[spirv(uniform, descriptor_set = 0, binding = 1)] color: &Vec3, out: &mut Vec4) {
	*out = vec4(color.x, color.y, color.z, 1.0);
}

// Texture shader

#[spirv(vertex)]
pub fn texture_vs(
	pos: Vec3,
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 1, binding = 0)] mvp: &Mat4,
	#[spirv(position)] out_pos: &mut Vec4,
	v_uv: &mut Vec2,
) {
	*out_pos = *mvp * pos.extend(1.0);
	*v_uv = uv;
}

#[spirv(fragment)]
pub fn texture_fs(
	uv: Vec2,
	#[spirv(descriptor_set = 0, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
	out: &mut Vec4,
) {
	*out = tex.sample(*sampler, uv);
}
