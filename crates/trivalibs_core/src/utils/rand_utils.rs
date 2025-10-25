use glam::{Vec3, Vec4, vec3, vec4};
use rand::random;

use crate::math::fit::Fit;

pub fn rand_range(min: f32, max: f32) -> f32 {
	min + (max - min) * random::<f32>()
}

pub fn rand_usize(max: usize) -> usize {
	(random::<f32>() * max as f32).floor() as usize
}

pub fn rand_u32(max: u32) -> u32 {
	(random::<f32>() * max as f32).floor() as u32
}

pub fn rand_range_u32(min: u32, max: u32) -> u32 {
	min + (random::<f32>() * (max - min) as f32).floor() as u32
}

pub fn rand_range_i32(min: i32, max: i32) -> i32 {
	min + (random::<f32>() * (max - min) as f32).floor() as i32
}

pub fn rand_f32() -> f32 {
	random::<f32>()
}

pub fn rand_f64() -> f64 {
	random::<f64>()
}

pub fn rand_bool() -> bool {
	random::<bool>()
}

pub fn rand_sign() -> f32 {
	if rand_f32() < 0.5 { 1.0 } else { -1.0 }
}

pub fn rand_vec3() -> Vec3 {
	vec3(random::<f32>(), random::<f32>(), random::<f32>())
}

pub fn rand_vec3_range(min: f32, max: f32) -> Vec3 {
	vec3(
		rand_range(min, max),
		rand_range(min, max),
		rand_range(min, max),
	)
}

pub fn rand_in_unit_sphere() -> Vec3 {
	loop {
		let p = vec3(
			rand_range(-1.0, 1.0),
			rand_range(-1.0, 1.0),
			rand_range(-1.0, 1.0),
		);

		let ls = p.length_squared();
		if ls < 1.0 && ls > 0.0000001 {
			return p;
		}
	}
}

pub fn rand_vec3_unit() -> Vec3 {
	rand_in_unit_sphere().normalize()
}

pub fn rand_vec4() -> Vec4 {
	vec4(
		random::<f32>(),
		random::<f32>(),
		random::<f32>(),
		random::<f32>(),
	)
}

pub fn rand_vec4_range(min: f32, max: f32) -> Vec4 {
	vec4(
		rand_range(min, max),
		rand_range(min, max),
		rand_range(min, max),
		rand_range(min, max),
	)
}

/// Returns a random number in the range [-1, 1] with normal distribution.
pub fn rand_normal() -> f32 {
	(random::<f32>() + random::<f32>() + random::<f32>()) / 1.5 - 1.0
}

/// Returns a random number in the range [0, 1] with normal distribution arround 0.5.
pub fn rand_normal_01() -> f32 {
	(random::<f32>() + random::<f32>() + random::<f32>()) / 3.
}

pub fn rand_normal_11() -> f32 {
	rand_normal_01().fit0111()
}

pub trait Pick<T> {
	fn pick(&self) -> &T;
}

impl<T> Pick<T> for &[T] {
	fn pick(&self) -> &T {
		&self[(random::<f64>() * self.len() as f64).floor() as usize]
	}
}

impl<T> Pick<T> for Vec<T> {
	fn pick(&self) -> &T {
		&self[(random::<f64>() * self.len() as f64).floor() as usize]
	}
}
