use glam::{vec3, Vec3};
use rand::random;

pub mod app_state;
pub mod f32;

pub fn default<T: Default>() -> T {
    std::default::Default::default()
}

pub fn random_range(min: f32, max: f32) -> f32 {
    min + (max - min) * random::<f32>()
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
