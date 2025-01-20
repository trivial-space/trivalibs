#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{vec4, Vec2, Vec4};
use spirv_std::{spirv, Image, Sampler};
#[allow(unused_imports)]
use trivalibs_shaders::blur::{gaussian_blur, gaussian_blur_13, gaussian_blur_9};

#[spirv(vertex)]
pub fn triangle_vs(pos: Vec2, uv: Vec2, #[spirv(position)] out_pos: &mut Vec4, out_uv: &mut Vec2) {
	*out_pos = pos.extend(0.0).extend(1.0);
	*out_uv = uv;
}

#[spirv(fragment)]
pub fn triangle_fs(uv: Vec2, out: &mut Vec4) {
	*out = vec4(uv.x, uv.y, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn blur_fs(
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
