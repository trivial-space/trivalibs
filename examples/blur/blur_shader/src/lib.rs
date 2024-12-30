#![no_std]

use spirv_std::glam::{Vec2, Vec4};
use spirv_std::{spirv, Image, Sampler};
use trivalibs_shaders::gaussian_blur;

#[spirv(fragment)]
pub fn frag(
	uv: Vec2,
	#[spirv(descriptor_set = 0, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
	#[spirv(uniform, descriptor_set = 1, binding = 0)] resolution: &Vec2,
	#[spirv(uniform, descriptor_set = 2, binding = 0)] dir: &Vec2,
	out: &mut Vec4,
) {
	*out = gaussian_blur(tex, sampler, 80.0, uv, *resolution, *dir);
}
