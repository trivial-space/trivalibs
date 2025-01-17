#![no_std]

use spirv_std::glam::{vec2, vec4, UVec2, Vec2, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::spirv;

#[spirv(fragment)]
pub fn main(
	coord: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	let tile_dim = vec2(200.0, 100.0);
	let size = vec2(size.x as f32, size.y as f32);
	let tile_size = size / tile_dim;
	let tile_ratio = tile_dim.x / tile_dim.y;
	let gap_size = 0.1;
	let gap = vec2(gap_size, gap_size * tile_ratio);
	let mut tile = coord * tile_size;
	tile.y -= (time * 0.5).fract() * 2.0;
	let y_offet = tile.y.ceil() % 2.0;
	if y_offet == 0.0 {
		tile.x += 0.5;
	}
	tile -= gap * 0.5;
	let tile = tile.fract();
	*out = if tile.x >= 1.0 - gap.x || tile.y >= 1.0 - gap.y {
		vec4(0.4, 0.6, 0.9, 1.0)
	} else {
		vec4(1.0, 0.8, 0.5, 1.0)
	}
}
