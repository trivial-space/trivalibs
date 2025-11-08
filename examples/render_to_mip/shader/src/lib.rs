#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::Vec3;
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::spirv;
use spirv_std::{
	glam::{swizzles::*, vec2, UVec2, Vec2, Vec4},
	Image, Sampler,
};
use trivalibs_nostd::float_ext::FloatExt;
use trivalibs_nostd::vec_ext::VecExt;

pub fn aspect_preserving_uv(uv: Vec2, size: UVec2) -> Vec2 {
	let aspect = size.x as f32 / size.y as f32;
	if aspect > 1.0 {
		uv * vec2(1.0, 1.0 / aspect)
	} else {
		uv * vec2(aspect, 1.0)
	}
}

#[spirv(fragment)]
pub fn image(
	coord: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] color: &Vec3,
	#[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
	#[spirv(descriptor_set = 1, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	out: &mut Vec4,
) {
	let coord = coord * 2.0;
	let idx = coord.floor();
	let coord = coord.frct();

	let color = if idx.x + idx.y == 1.0 {
		*color
	} else {
		let col = tex.sample(*sampler, coord);
		col.xyz()
	};

	*out = color.extend(1.0);
}

#[spirv(fragment)]
pub fn mip_sampling(
	coord: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] time: &f32,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] mips: &f32,
	#[spirv(descriptor_set = 0, binding = 2)] sampler: &Sampler,
	#[spirv(descriptor_set = 1, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	out: &mut Vec4,
) {
	let col = tex.sample_by_lod(*sampler, coord, (*time * 0.1).frct() * mips);
	*out = col;
}
