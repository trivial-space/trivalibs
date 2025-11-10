#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{Vec2, Vec4, Vec4Swizzles};
use spirv_std::{Image, Sampler, spirv};

/// Fragment shader that samples a texture.
/// This is used for the display layer that switches between red/blue textures.
#[spirv(fragment)]
pub fn tex_fs(
	uv: Vec2,
	#[spirv(descriptor_set = 0, binding = 0)] sampler: &Sampler,
	#[spirv(descriptor_set = 1, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	out: &mut Vec4,
) {
	*out = tex.sample(*sampler, uv);
}

/// Fragment shader that outputs a solid color from a uniform.
/// This is used to create the red and blue layers.
#[spirv(fragment)]
pub fn col_fs(
	_uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] color: &Vec4,
	out: &mut Vec4,
) {
	*out = color.xyz().extend(1.0);
}
