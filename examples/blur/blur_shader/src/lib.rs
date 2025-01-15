#![no_std]

use spirv_std::glam::{Vec2, Vec4};
use spirv_std::{spirv, Image, Sampler};
#[allow(unused_imports)]
use trivalibs_shaders::{gaussian_blur, gaussian_blur_13, gaussian_blur_9};

#[spirv(fragment)]
pub fn frag(
	uv: Vec2,
	#[spirv(descriptor_set = 0, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
	#[spirv(uniform, descriptor_set = 1, binding = 0)] diameter: &f32,
	#[spirv(uniform, descriptor_set = 1, binding = 1)] resolution: &Vec2,
	#[spirv(uniform, descriptor_set = 1, binding = 2)] dir: &Vec2,
	out: &mut Vec4,
) {
	// *out = gaussian_blur(tex, sampler, *radius, uv, *resolution, *dir);
	*out = gaussian_blur_9(tex, sampler, uv, *resolution, *dir * *diameter);
}
