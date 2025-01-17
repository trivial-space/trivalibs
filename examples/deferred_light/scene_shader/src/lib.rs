#![no_std]

use spirv_std::glam::{Mat4, Quat, Vec3, Vec4, Vec4Swizzles};
use spirv_std::spirv;

#[spirv(vertex)]
pub fn vertex(
	position: Vec3,
	color: Vec3,
	normal: Vec3,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] model_mat: &Mat4,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] vp_mat: &Mat4,
	#[spirv(uniform, descriptor_set = 0, binding = 2)] normal_rot: &Quat,
	#[spirv(position)] clip_pos: &mut Vec4,
	out_pos: &mut Vec3,
	out_color: &mut Vec3,
	out_norm: &mut Vec3,
	out_depth: &mut f32,
) {
	*out_color = color;
	*out_norm = normal_rot.mul_vec3(normal);
	let pos = *model_mat * position.extend(1.0);
	*out_pos = pos.xyz();
	*clip_pos = *vp_mat * pos;
	*out_depth = clip_pos.z;
}

#[spirv(fragment)]
pub fn fragment(
	in_pos: Vec3,
	in_color: Vec3,
	in_norm: Vec3,
	in_depth: f32,
	frag_color: &mut Vec4,
	frag_norm: &mut Vec4,
	frag_pos: &mut Vec4,
) {
	*frag_color = in_color.extend(1.0);
	*frag_norm = in_norm.extend(0.0);
	*frag_pos = in_pos.extend(in_depth);
}
