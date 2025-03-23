use core::f32::consts::PI;

use crate::{
	fit::Fit,
	smoothstep::{Smoothen, SmoothenMore},
	step::Step,
};
use spirv_std::glam::{vec3, vec4, Vec3};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

pub fn rgb2hsl(c: Vec3) -> Vec3 {
	let k = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
	let p = vec4(c.z, c.y, k.w, k.z).lerp(vec4(c.y, c.z, k.x, k.y), c.z.step(c.y));
	let q = vec4(p.x, p.y, p.w, c.x).lerp(vec4(c.x, p.y, p.z, p.x), p.x.step(c.x));
	let d = q.x - q.w.min(q.y);
	let e = 1.0e-10;
	vec3(
		(q.z + (q.w - q.y) / (6.0 * d + e)).abs(),
		d / (q.x + e),
		q.x,
	)
}

//  Function from Iñigo Quiles
//  https://www.shadertoy.com/view/MsS3Wc

pub fn hsv2rgb(c: Vec3) -> Vec3 {
	let rgb = ((((c.x * 6.0 + vec3(0.0, 4.0, 2.0)) % 6.0) - 3.0).abs() - 1.0)
		.clamp(Vec3::ZERO, Vec3::ONE);
	c.z * Vec3::ONE.lerp(rgb, c.y)
}

pub fn hsv2rgb_smooth(c: Vec3) -> Vec3 {
	let rgb = ((((c.x * 6.0 + vec3(0.0, 4.0, 2.0)) % 6.0) - 3.0).abs() - 1.0).smoothen();
	c.z * Vec3::ONE.lerp(rgb, c.y)
}

pub fn hsv2rgb_smoother(c: Vec3) -> Vec3 {
	let rgb = ((((c.x * 6.0 + vec3(0.0, 4.0, 2.0)) % 6.0) - 3.0).abs() - 1.0).smoothen_more();
	c.z * Vec3::ONE.lerp(rgb, c.y)
}

/// blend rgb color using trigonometry. This is an experiment. hsv2rgb_smooth looks almost the same, but is much cheeper.
pub fn hsv2rgb_smoothest(c: Vec3) -> Vec3 {
	let mut rgb = ((((c.x * 6.0 + vec3(0.0, 4.0, 2.0)) % 6.0) - 3.0).abs() - 1.0)
		.clamp(Vec3::ZERO, Vec3::ONE);

	rgb.x = ((rgb.x + 1.0) * PI).cos().fit1101();
	rgb.y = ((rgb.y + 1.0) * PI).cos().fit1101();
	rgb.z = ((rgb.z + 1.0) * PI).cos().fit1101();

	c.z * Vec3::ONE.lerp(rgb, c.y)
}
