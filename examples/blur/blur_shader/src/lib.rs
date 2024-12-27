#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{vec2, Vec2, Vec4};
use spirv_std::{spirv, Image, Sampler};

pub fn blur13(
	image: &Image!(2D, type=f32, sampled),
	sampler: &Sampler,
	uv: Vec2,
	res: Vec2,
	dir: Vec2,
) -> Vec4 {
	let mut color = Vec4::ZERO;
	let off1 = Vec2::splat(1.411764705882353) * dir;
	let off2 = Vec2::splat(3.2941176470588234) * dir;
	let off3 = Vec2::splat(5.176470588235294) * dir;
	color += image.sample(*sampler, uv) * 0.1964825501511404;
	color += image.sample(*sampler, uv + (off1 / res)) * 0.2969069646728344;
	color += image.sample(*sampler, uv - (off1 / res)) * 0.2969069646728344;
	color += image.sample(*sampler, uv + (off2 / res)) * 0.09447039785044732;
	color += image.sample(*sampler, uv - (off2 / res)) * 0.09447039785044732;
	color += image.sample(*sampler, uv + (off3 / res)) * 0.010381362401148057;
	color += image.sample(*sampler, uv - (off3 / res)) * 0.010381362401148057;
	color
}

#[spirv(fragment)]
pub fn frag(
	uv: Vec2,
	#[spirv(descriptor_set = 0, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
	#[spirv(uniform, descriptor_set = 1, binding = 0)] resolution: &Vec2,
	out: &mut Vec4,
) {
	*out = blur13(tex, sampler, uv, *resolution, vec2(1.0, 0.0));
	*out += blur13(tex, sampler, uv, *resolution, vec2(0.0, 1.0));
}
