#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{vec3, Mat4, Vec2, Vec3, Vec4};
use spirv_std::spirv;

#[spirv(vertex)]
pub fn ground_vert(
	position: Vec3,
	uv: Vec2,
	normal: Vec3,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] mvp_mat: &Mat4,
	#[spirv(position)] out_pos: &mut Vec4,
	out_norm: &mut Vec3,
	out_uv: &mut Vec2,
) {
	*out_pos = *mvp_mat * position.extend(1.0);
	*out_norm = normal;
	*out_uv = uv;
}

#[spirv(fragment)]
pub fn ground_frag(_in_norm: Vec3, in_uv: Vec2, out: &mut Vec4) {
	let uv = in_uv * 40.0;
	let uv = uv.fract();

	let col = if uv.x < 0.05 || uv.y < 0.05 {
		Vec3::splat(0.4)
	} else {
		vec3(in_uv.x, in_uv.y, 0.5)
	};

	*out = col.powf(2.2).extend(1.0);
}
