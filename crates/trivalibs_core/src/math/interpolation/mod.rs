use glam::{Vec2, Vec3};

pub trait Interpolate {
	/// Linear interpolation.
	fn lerp(t: f32, a: Self, b: Self) -> Self;

	/// Cosine interpolation.
	fn cosine(t: f32, a: Self, b: Self) -> Self;

	/// Quadratic Bézier interpolation.
	///
	/// `a` is the first point; `b` is the second point and `u` is the tangent of `a` to the curve.
	fn quadratic_bezier(t: f32, a: Self, u: Self, b: Self) -> Self;

	/// Cubic Bézier interpolation.
	///
	/// `a` is the first point; `b` is the second point; `u` is the output tangent of `a` to the curve and `v` is the
	/// input tangent of `b` to the curve.
	fn cubic_bezier(t: f32, a: Self, u: Self, v: Self, b: Self) -> Self;
}

#[macro_export]
macro_rules! impl_Interpolate {
	($v:ty) => {
		impl $crate::math::interpolation::Interpolate for $v {
			fn lerp(t: f32, a: Self, b: Self) -> Self {
				a * (1. - t) + b * t
			}

			fn cosine(t: f32, a: Self, b: Self) -> Self {
				let cos_nt = (1. - (t * std::f32::consts::PI).cos()) * 0.5;
				<Self as $crate::math::interpolation::Interpolate>::lerp(cos_nt, a, b)
			}

			fn quadratic_bezier(t: f32, a: Self, u: Self, b: Self) -> Self {
				let one_t = 1. - t;
				let one_t2 = one_t * one_t;

				u + (a - u) * one_t2 + (b - u) * t * t
			}

			fn cubic_bezier(t: f32, a: Self, u: Self, v: Self, b: Self) -> Self {
				let one_t = 1. - t;
				let one_t2 = one_t * one_t;
				let one_t3 = one_t2 * one_t;
				let t2 = t * t;

				a * one_t3 + (u * one_t2 * t + v * one_t * t2) * 3. + b * t2 * t
			}
		}
	};
}

impl_Interpolate!(f32);
impl_Interpolate!(Vec2);
impl_Interpolate!(Vec3);
