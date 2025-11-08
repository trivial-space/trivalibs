#![no_std]
#![allow(unexpected_cfgs)]

#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::spirv;
use spirv_std::{
	glam::{vec2, vec3, Mat2, UVec2, Vec2, Vec4},
	Image, Sampler,
};
use trivalibs_nostd::float_ext::FloatExt;
use trivalibs_nostd::{color::hsv2rgb, random::hash::hash, vec_ext::VecExt};

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
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	let uv = aspect_preserving_uv(coord, *size).fit0111();
	let rot_mat = Mat2::from_angle(*time * 0.05);
	let uv = rot_mat * uv;
	let uv = uv * (7.0 + (*time * 0.1).sin());
	let mut idx = uv.floor();
	let mut uv = uv.frct();

	for i in 1..=3 {
		let h1 = hash((idx.x + (8.0 * i as f32) * idx.y + 8.0.powf(i as f32 + 1.0)) as u32);

		let should_subdivide = h1 < 0.5;
		let subdivision_dir_x = h1 < 0.25;

		if should_subdivide {
			uv = if subdivision_dir_x {
				vec2(uv.x * 2.0, uv.y)
			} else {
				vec2(uv.x, uv.y * 2.0)
			};

			idx = uv.floor() * 8.0 + idx;
			uv = uv.frct();

			let h2 = hash((idx.x + (8.0 * (1 + i) as f32) * idx.y + (8.0.powf(i as f32))) as u32);

			let should_subdivide = h2 < 0.5;

			if should_subdivide {
				uv = if subdivision_dir_x {
					vec2(uv.x, uv.y * 2.0)
				} else {
					vec2(uv.x * 2.0, uv.y)
				};

				idx = uv.floor() * (8.0 * (i + 2) as f32) + idx;
				uv = uv.frct();
			}
		}
	}

	let h = hash(((idx.x + (8.0 * 4.0) * idx.y) + 9.0.powf(4.0f32)) as u32);

	let color = hsv2rgb(vec3(h, 0.7, 0.7));

	*out = color.powf(2.2).extend(1.0);
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
	let col = tex.sample_by_lod(*sampler, coord, (*time * 0.2).sin().fit1101() * mips);
	*out = col;
}

#[spirv(fragment)]
pub fn wave_effect(
	coord: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] time: &f32,
	#[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
	#[spirv(descriptor_set = 1, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	out: &mut Vec4,
) {
	let coord = vec2(coord.x + (coord.y * 30.0 + time).sin() * 0.005, coord.y);
	let col = tex.sample(*sampler, coord);
	*out = col;
}
