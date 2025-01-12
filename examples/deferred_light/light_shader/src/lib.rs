#![no_std]

use spirv_std::glam::{swizzles::*, Vec2, Vec3, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::{spirv, Image, Sampler};

#[spirv(fragment)]
pub fn fragment(
	in_uv: Vec2,
	#[spirv(descriptor_set = 0, binding = 0)] color_tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 0, binding = 1)] color_sampler: &Sampler,
	#[spirv(descriptor_set = 1, binding = 0)] normal_tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 1, binding = 1)] normal_sampler: &Sampler,
	#[spirv(descriptor_set = 2, binding = 0)] pos_tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 2, binding = 1)] pos_sampler: &Sampler,

	// #[spirv(uniform, descriptor_set = 3, binding = 0)] eye_pos: &Vec3,
	#[spirv(uniform, descriptor_set = 3, binding = 0)] light_pos: &Vec3,
	// #[spirv(uniform, descriptor_set = 5, binding = 0)] light_color: &Vec3,
	frag_color: &mut Vec4,
) {
	let light_color = Vec3::new(1.0, 1.0, 1.0);
	let eye_pos = Vec3::new(0.0, 5.0, 0.0);
	let color = color_tex.sample(*color_sampler, in_uv);
	let normal = normal_tex.sample(*normal_sampler, in_uv).xyz();
	let pos = pos_tex.sample(*pos_sampler, in_uv);

	let light_dir = (*light_pos - pos.xyz()).normalize();
	// let normal = normal.xyz() * 2.0 - 1.0;
	let view_dir = (eye_pos - pos.xyz()).normalize();
	let half_dir = (light_dir + view_dir).normalize();

	// Half Lambert diffuse
	let n_dot_l = normal.dot(light_dir) * 0.5 + 0.5;
	let diffuse = light_color * n_dot_l;

	// Specular (Blinn-Phong)
	let n_dot_h = normal.dot(half_dir).max(0.0);
	let specular = light_color * n_dot_h.powf(32.0);

	// let final_color = (color.xyz() * (1.0 - (pos.w / 25.0))).extend(1.0);
	let final_color = color.xyz() * diffuse + specular;
	*frag_color = final_color.extend(1.0);
}
