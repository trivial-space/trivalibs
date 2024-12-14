//! Ported to Rust from <https://github.com/Tw1ddle/Sky-Shader/blob/master/src/shaders/glsl/sky.fragment>
#![allow(unexpected_cfgs)]
#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::glam::{vec2, vec4, Vec2, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::spirv;

#[spirv(fragment)]
pub fn main_fs(uv: Vec2, output: &mut Vec4) {
	let tile_size = vec2(6.0, 12.0);
	let gap_size = tile_size * 0.02;
	let mut tile = uv * tile_size;
	let y_offet = tile.y.floor() % 2.0;
	if y_offet == 1.0 {
		tile.x += 0.5;
	}
	tile -= gap_size * 0.5;
	let tile = tile - tile.floor();
	*output = if tile.x >= 1.0 - gap_size.x || tile.y >= 1.0 - gap_size.y {
		vec4(0.4, 0.6, 0.9, 1.0)
	} else {
		vec4(1.0, 0.8, 0.5, 1.0)
	}
}

#[spirv(vertex)]
pub fn main_vs(
	#[spirv(vertex_index)] vert_idx: i32,
	#[spirv(position)] builtin_pos: &mut Vec4,
	uv: &mut Vec2,
) {
	*uv = vec2(((vert_idx << 1) & 2) as f32, (vert_idx & 2) as f32);
	let pos = *uv * 2.0 - Vec2::ONE;
	*builtin_pos = pos.extend(0.0).extend(1.0);
}
