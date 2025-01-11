#![no_std]

use spirv_std::glam::{Vec2, Vec4};
use spirv_std::{spirv, Image, Sampler};

#[spirv(fragment)]
pub fn fragment(
	in_uv: Vec2,
	#[spirv(descriptor_set = 0, binding = 0)] color_tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 0, binding = 1)] color_sampler: &Sampler,
	#[spirv(descriptor_set = 1, binding = 0)] _normal_tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 1, binding = 1)] _normal_sampler: &Sampler,
	#[spirv(descriptor_set = 2, binding = 0)] _pos_tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 2, binding = 1)] _pos_sampler: &Sampler,
	frag_color: &mut Vec4,
) {
	*frag_color = color_tex.sample(*color_sampler, in_uv);
}
