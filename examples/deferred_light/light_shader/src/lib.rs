#![no_std]

use spirv_std::glam::{swizzles::*, Vec2, Vec3, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::{spirv, Image, Sampler};

#[spirv(fragment)]
pub fn fragment(
	in_uv: Vec2,
	#[spirv(descriptor_set = 0, binding = 0)] color_tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 0, binding = 1)] normal_tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 0, binding = 2)] pos_tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 0, binding = 3)] sampler: &Sampler,
	#[spirv(uniform, descriptor_set = 0, binding = 4)] eye_pos: &Vec3,
	#[spirv(uniform, descriptor_set = 0, binding = 5)] light_pos: &Vec3,
	#[spirv(uniform, descriptor_set = 0, binding = 6)] light_color: &Vec3,
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
	let specular = *light_color * n_dot_h.powf(30.0);

	// let final_color = (color.xyz() * (1.0 - (pos.w / 25.0))).extend(1.0);
	let final_color = color * diffuse + specular + color * 0.01;
	// let final_color = Vec3::splat(1.0) * diffuse + specular;
	*frag_color = final_color.extend(1.0);
}
