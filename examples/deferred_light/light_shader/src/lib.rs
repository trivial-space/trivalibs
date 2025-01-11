#![no_std]

use spirv_std::glam::{swizzles::*, Vec2, Vec4};
use spirv_std::{spirv, Image, Sampler};

#[spirv(fragment)]
pub fn fragment(
	in_uv: Vec2,
	#[spirv(descriptor_set = 0, binding = 0)] color_tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 0, binding = 1)] color_sampler: &Sampler,
	#[spirv(descriptor_set = 1, binding = 0)] normal_tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 1, binding = 1)] normal_sampler: &Sampler,
	#[spirv(descriptor_set = 2, binding = 0)] pos_tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 2, binding = 1)] pos_sampler: &Sampler,
	frag_color: &mut Vec4,
) {
	let color = color_tex.sample(*color_sampler, in_uv);
	let _normal = normal_tex.sample(*normal_sampler, in_uv);
	let pos = pos_tex.sample(*pos_sampler, in_uv);
	*frag_color = (color.xyz() * (1.0 - (pos.w / 25.0))).extend(1.0);
}
