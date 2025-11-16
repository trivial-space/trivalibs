#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{Vec2, Vec3, Vec4};
use spirv_std::spirv;

#[spirv(vertex)]
pub fn vertex(position: Vec2, #[spirv(position)] clip_pos: &mut Vec4) {
	*clip_pos = position.extend(0.0).extend(1.0);
}

#[spirv(fragment)]
pub fn fragment(
	#[spirv(uniform, descriptor_set = 0, binding = 0)] color: &Vec3,
	frag_color: &mut Vec4,
) {
	*frag_color = color.powf(2.2).extend(1.0);
}
