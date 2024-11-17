use glam::{vec3, Vec3};
use rand::random;

pub fn random_range(min: f32, max: f32) -> f32 {
	min + (max - min) * random::<f32>()
}

pub fn rand_int(max: usize) -> usize {
	(random::<f32>() * max as f32).floor() as usize
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

pub fn random_in_unit_sphere() -> Vec3 {
	loop {
		let p = vec3(
			random_range(-1.0, 1.0),
			random_range(-1.0, 1.0),
			random_range(-1.0, 1.0),
		);

		let ls = p.length_squared();
		if ls < 1.0 && ls > 0.0000001 {
			return p;
		}
	}
}

/// Returns a random number in the range [-1, 1] with normal distribution.
pub fn random_normal() -> f32 {
	(random::<f32>() + random::<f32>() + random::<f32>()) / 1.5 - 1.0
}

/// Returns a random number in the range [0, 1] with normal distribution arround 0.5.
pub fn random_normal_01() -> f32 {
	(random::<f32>() + random::<f32>() + random::<f32>()) / 3.
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
