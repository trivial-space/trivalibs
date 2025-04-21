#![no_std]
#![allow(unexpected_cfgs)]

#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::{
	glam::{uvec2, vec2, vec3, UVec2, Vec2, Vec3, Vec4},
	spirv,
};
use trivalibs_shaders::{
	bits::FloatBits,
	float_ext::FloatExt,
	random::{
		hash::{hash, hash21, hash2d, hash3d, hashi},
		simplex::{
			simplex_noise_2d, simplex_noise_3d, simplex_noise_4d, tiling_rot_noise_2d,
			tiling_rot_noise_3d, tiling_simplex_noise_2d,
		},
	},
};

const GAMMA: f32 = 2.2;

fn aspect_preserving_uv(uv: Vec2, size: UVec2) -> Vec2 {
	let aspect = size.x as f32 / size.y as f32;
	if aspect > 1.0 {
		uv * vec2(1.0, 1.0 / aspect)
	} else {
		uv * vec2(aspect, 1.0)
	}
}

#[spirv(fragment)]
pub fn simplex_3d_shader(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	let uv = aspect_preserving_uv(uv, *size);
	let uv = uv * 10.0;

	let noise = simplex_noise_3d(uv.extend(*time)).fit1101();

	let color = Vec3::splat(noise).powf(GAMMA).extend(1.0);
	*out = color;
}

#[spirv(fragment)]
pub fn simplex_2d_shader(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	let uv = aspect_preserving_uv(uv, *size);

	let noise = simplex_noise_2d(uv * (time.sin().fit1101() * 20. + 0.5)).fit1101();

	let color = Vec3::splat(noise).powf(GAMMA).extend(1.0);
	*out = color;
}

#[spirv(fragment)]
pub fn simplex_4d_shader(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	let uv = aspect_preserving_uv(uv, *size);
	let uv = uv * 10.0;

	let noise = simplex_noise_4d(uv.extend(123. + *time * 0.2345).extend(*time)).fit1101();

	let color = Vec3::splat(noise).powf(GAMMA).extend(1.0);
	*out = color;
}

#[spirv(fragment)]
pub fn tiling_simplex_shader(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	let uv = aspect_preserving_uv(uv, *size);

	let uv = (uv * 1.5 + *time * 0.1).fract();
	let scale = (time * 0.2).sin().fit1101() * 4. + 0.5;

	let noise = tiling_simplex_noise_2d(uv, scale).fit1101();

	let color = Vec3::splat(noise).powf(GAMMA).extend(1.0);
	*out = color;
}

#[spirv(fragment)]
pub fn tiling_noise_2d_shader(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	let uv = aspect_preserving_uv(uv, *size);

	let noise = tiling_rot_noise_2d(
		(uv * 2.5).fract() * 4. + 0.5, // shift by 0.5 to avoid tiling artifacts
		vec2(1.0, 1.0) * 4.,
		*time * 0.8,
	)
	.0
	.fit1101();

	let color = Vec3::splat(noise).powf(GAMMA).extend(1.0);
	*out = color;
}

#[spirv(fragment)]
pub fn tiling_noise_3d_shader(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	let uv = aspect_preserving_uv(uv, *size);

	let noise = tiling_rot_noise_3d(
		((uv * 2.5).fract() * 4. + 0.5) // shift by 0.5 to avoid tiling artifacts
			.extend(*time * 0.2),
		Vec3::ONE * 4.,
		*time * 0.3345,
	)
	.0
	.fit1101();

	let color = Vec3::splat(noise).powf(GAMMA).extend(1.0);
	*out = color;
}

#[spirv(fragment)]
pub fn hash_shader(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] _size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = {
		let time = *time;
		let q_uv = (uv * 2.).fract();
		let q_idx = (uv * 2.).floor().as_uvec2();

		let color = if uv.x > 0.98 || uv.y > 0.98 || uv.x < 0.02 || uv.y < 0.02 {
			Vec3::ZERO
		} else if q_uv.x > 0.98 || q_uv.y > 0.98 || q_uv.x < 0.02 || q_uv.y < 0.02 {
			Vec3::ZERO
		} else if q_idx.eq(&uvec2(0, 0)) {
			let v = hash(q_uv.x.to_bits() + hashi((q_uv.y + time).to_bits()));
			vec3(v, 0.0, 0.0)
		} else if q_idx.eq(&uvec2(1, 0)) {
			let v = hash21((q_uv + time).to_bits());
			vec3(0.0, v, 0.0)
		} else if q_idx.eq(&uvec2(0, 1)) {
			let v = hash2d((q_uv + time).to_bits());
			v.extend(1.0)
		} else if q_idx.eq(&uvec2(1, 1)) {
			hash3d(q_uv.extend(time).to_bits())
		} else {
			vec3(0.0, 1.0, 1.0)
		};

		color.powf(GAMMA).extend(1.0)
	};
}
