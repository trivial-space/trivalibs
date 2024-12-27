#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{vec4, Vec2, Vec4};
use spirv_std::spirv;

#[spirv(vertex)]
pub fn vert(pos: Vec2, uv: Vec2, #[spirv(position)] out_pos: &mut Vec4, out_uv: &mut Vec2) {
	*out_pos = pos.extend(0.0).extend(1.0);
	*out_uv = uv;
}

#[spirv(fragment)]
pub fn frag(uv: Vec2, out: &mut Vec4) {
	*out = vec4(uv.x, uv.y, 0.0, 1.0);
}
