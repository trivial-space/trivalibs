#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{Vec2, Vec3, Vec4, vec4};
use spirv_std::spirv;
use trivalibs_nostd::prelude::*;

#[spirv(vertex)]
pub fn vertex(position: Vec2, #[spirv(position)] clip_pos: &mut Vec4, pos: &mut Vec2) {
	*clip_pos = position.extend(0.0).extend(1.0);
	*pos = position.fit1101();
}

#[spirv(fragment)]
pub fn fragment(
	pos: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] color: &Vec3,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] is_vertical: &u32,
	frag_color: &mut Vec4,
) {
	let alpha = if *is_vertical != 0 {
		if (pos.y / 0.01) as u32 % 2 == 0 {
			1.0
		} else {
			0.0
		}
	} else {
		if (pos.x / 0.01) as u32 % 2 == 0 {
			1.0
		} else {
			0.0
		}
	};
	*frag_color = color.powf(2.2).extend(alpha);
}

#[spirv(fragment)]
pub fn effect_fragment(
	uv: Vec2,
	#[spirv(descriptor_set = 0, binding = 0)] sampler: &spirv_std::Sampler,
	#[spirv(descriptor_set = 1, binding = 0)] texture: &spirv_std::Image!(2D, type=f32, sampled),
	frag_color: &mut Vec4,
) {
	let color: Vec4 = texture.sample(*sampler, uv);
	*frag_color = vec4(color.x, color.y, color.z, color.w);
}
