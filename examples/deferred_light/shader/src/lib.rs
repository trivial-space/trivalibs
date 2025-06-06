#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{swizzles::*, Mat4, Quat, Vec2, Vec3, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::{spirv, Image, Sampler};

#[spirv(vertex)]
pub fn scene_vs(
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
pub fn scene_fs(
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

#[spirv(fragment)]
pub fn light_fs(
	in_uv: Vec2,
	#[spirv(descriptor_set = 0, binding = 0)] sampler: &Sampler,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] eye_pos: &Vec3,
	#[spirv(uniform, descriptor_set = 0, binding = 2)] light_pos: &Vec3,
	#[spirv(uniform, descriptor_set = 0, binding = 3)] light_color: &Vec3,
	#[spirv(descriptor_set = 1, binding = 0)] color_tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 1, binding = 1)] normal_tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 1, binding = 2)] pos_tex: &Image!(2D, type=f32, sampled),
	frag_color: &mut Vec4,
) {
	let color = color_tex.sample(*sampler, in_uv).xyz();
	let normal = normal_tex.sample(*sampler, in_uv).xyz().normalize();
	let pos = pos_tex.sample(*sampler, in_uv).xyz();

	let light_dir = (*light_pos - pos).normalize();
	let view_dir = (*eye_pos - pos).normalize();
	let half_dir = (light_dir + view_dir).normalize();

	// Half Lambert diffuse
	let n_dot_l = normal.dot(light_dir);
	let diffuse = *light_color * n_dot_l.max(0.0);

	// Specular (Blinn-Phong)
	let n_dot_h = normal.dot(half_dir).max(0.0);
	let specular = *light_color * n_dot_h.powf(30.0) * 1.8;

	// let final_color = (color.xyz() * (1.0 - (pos.w / 25.0))).extend(1.0);
	let mut final_color = color * diffuse + specular + color * 0.01;
	final_color *= 0.4;
	// let final_color = Vec3::splat(1.0) * diffuse + specular;
	*frag_color = final_color.extend(1.0);
}
