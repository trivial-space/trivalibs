#![allow(unexpected_cfgs)]

use crate::float_ext::FloatExt;
#[cfg(not(target_arch = "spirv"))]
use glam::{vec2, vec3, vec4, Vec2, Vec3, Vec4};
#[cfg(target_arch = "spirv")]
use spirv_std::glam::{vec2, vec3, vec4, Vec2, Vec3, Vec4};
#[cfg(target_arch = "spirv")]
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

pub trait VecExt
where
	Self: Sized,
{
	fn sin(self) -> Self;
	fn cos(self) -> Self;
	fn sqrt(self) -> Self;
	fn frct(self) -> Self;

	fn fit0111(self) -> Self;
	fn fit1101(self) -> Self;
	fn clamp01(self) -> Self;

	fn step(self, edge: Self) -> Self;
	fn step_f32(self, edge: f32) -> Self;
	fn step_fn<F: Fn(f32) -> f32>(self, edge0: Self, edge1: Self, f: F) -> Self;

	fn smoothen(self) -> Self;
	fn smoothen_more(self) -> Self;
	fn smoothstep(self, edge0: Self, edge1: Self) -> Self;

	fn lerp_vec(self, other: Self, t: Self) -> Self;
}

impl VecExt for Vec2 {
	fn sin(self) -> Self {
		vec2(self.x.sin(), self.y.sin())
	}
	fn cos(self) -> Self {
		vec2(self.x.cos(), self.y.cos())
	}
	fn sqrt(self) -> Self {
		vec2(self.x.sqrt(), self.y.sqrt())
	}
	fn frct(self) -> Self {
		vec2(self.x.frct(), self.y.frct())
	}

	fn fit0111(self) -> Self {
		vec2(self.x.fit0111(), self.y.fit0111())
	}
	fn fit1101(self) -> Self {
		vec2(self.x.fit1101(), self.y.fit1101())
	}
	fn clamp01(self) -> Self {
		self.clamp(Vec2::ZERO, Vec2::ONE)
	}

	fn step(self, edge: Vec2) -> Vec2 {
		vec2(self.x.step(edge.x), self.y.step(edge.y))
	}
	fn step_f32(self, edge: f32) -> Vec2 {
		vec2(self.x.step(edge), self.y.step(edge))
	}
	fn step_fn<F: Fn(f32) -> f32>(self, edge0: Self, edge1: Self, f: F) -> Self {
		vec2(
			self.x.step_fn(edge0.x, edge1.x, &f),
			self.y.step_fn(edge0.y, edge1.y, &f),
		)
	}

	fn smoothen(self) -> Self {
		vec2(self.x.smoothen(), self.y.smoothen())
	}
	fn smoothen_more(self) -> Self {
		vec2(self.x.smoothen_more(), self.y.smoothen_more())
	}
	fn smoothstep(self, edge0: Self, edge1: Self) -> Self {
		vec2(
			self.x.smoothstep(edge0.x, edge1.x),
			self.y.smoothstep(edge0.y, edge1.y),
		)
	}

	fn lerp_vec(self, other: Self, t: Self) -> Self {
		vec2(self.x.lerp(other.x, t.x), self.y.lerp(other.y, t.y))
	}
}

impl VecExt for Vec3 {
	fn sin(self) -> Self {
		vec3(self.x.sin(), self.y.sin(), self.z.sin())
	}
	fn cos(self) -> Self {
		vec3(self.x.cos(), self.y.cos(), self.z.cos())
	}
	fn sqrt(self) -> Self {
		vec3(self.x.sqrt(), self.y.sqrt(), self.z.sqrt())
	}
	fn frct(self) -> Self {
		vec3(self.x.frct(), self.y.frct(), self.z.frct())
	}

	fn fit0111(self) -> Self {
		vec3(self.x.fit0111(), self.y.fit0111(), self.z.fit0111())
	}
	fn fit1101(self) -> Self {
		vec3(self.x.fit1101(), self.y.fit1101(), self.z.fit1101())
	}
	fn clamp01(self) -> Self {
		self.clamp(Vec3::ZERO, Vec3::ONE)
	}

	fn step(self, edge: Vec3) -> Vec3 {
		vec3(
			self.x.step(edge.x),
			self.y.step(edge.y),
			self.z.step(edge.z),
		)
	}
	fn step_f32(self, edge: f32) -> Vec3 {
		vec3(self.x.step(edge), self.y.step(edge), self.z.step(edge))
	}
	fn step_fn<F: Fn(f32) -> f32>(self, edge0: Self, edge1: Self, f: F) -> Self {
		vec3(
			self.x.step_fn(edge0.x, edge1.x, &f),
			self.y.step_fn(edge0.y, edge1.y, &f),
			self.z.step_fn(edge0.z, edge1.z, &f),
		)
	}

	fn smoothen(self) -> Self {
		vec3(self.x.smoothen(), self.y.smoothen(), self.z.smoothen())
	}
	fn smoothen_more(self) -> Self {
		vec3(
			self.x.smoothen_more(),
			self.y.smoothen_more(),
			self.z.smoothen_more(),
		)
	}
	fn smoothstep(self, edge0: Self, edge1: Self) -> Self {
		vec3(
			self.x.smoothstep(edge0.x, edge1.x),
			self.y.smoothstep(edge0.y, edge1.y),
			self.z.smoothstep(edge0.z, edge1.z),
		)
	}

	fn lerp_vec(self, other: Self, t: Self) -> Self {
		vec3(
			self.x.lerp(other.x, t.x),
			self.y.lerp(other.y, t.y),
			self.z.lerp(other.z, t.z),
		)
	}
}

impl VecExt for Vec4 {
	fn sin(self) -> Self {
		vec4(self.x.sin(), self.y.sin(), self.z.sin(), self.w.sin())
	}
	fn cos(self) -> Self {
		vec4(self.x.cos(), self.y.cos(), self.z.cos(), self.w.cos())
	}
	fn sqrt(self) -> Self {
		vec4(self.x.sqrt(), self.y.sqrt(), self.z.sqrt(), self.w.sqrt())
	}
	fn frct(self) -> Self {
		vec4(self.x.frct(), self.y.frct(), self.z.frct(), self.w.frct())
	}

	fn fit0111(self) -> Self {
		vec4(
			self.x.fit0111(),
			self.y.fit0111(),
			self.z.fit0111(),
			self.w.fit0111(),
		)
	}
	fn fit1101(self) -> Self {
		vec4(
			self.x.fit1101(),
			self.y.fit1101(),
			self.z.fit1101(),
			self.w.fit1101(),
		)
	}
	fn clamp01(self) -> Self {
		self.clamp(Vec4::ZERO, Vec4::ONE)
	}

	fn step(self, edge: Vec4) -> Vec4 {
		vec4(
			self.x.step(edge.x),
			self.y.step(edge.y),
			self.z.step(edge.z),
			self.w.step(edge.w),
		)
	}
	fn step_f32(self, edge: f32) -> Vec4 {
		vec4(
			self.x.step(edge),
			self.y.step(edge),
			self.z.step(edge),
			self.w.step(edge),
		)
	}
	fn step_fn<F: Fn(f32) -> f32>(self, edge0: Self, edge1: Self, f: F) -> Self {
		vec4(
			self.x.step_fn(edge0.x, edge1.x, &f),
			self.y.step_fn(edge0.y, edge1.y, &f),
			self.z.step_fn(edge0.z, edge1.z, &f),
			self.w.step_fn(edge0.w, edge1.w, &f),
		)
	}

	fn smoothen(self) -> Self {
		vec4(
			self.x.smoothen(),
			self.y.smoothen(),
			self.z.smoothen(),
			self.w.smoothen(),
		)
	}
	fn smoothen_more(self) -> Self {
		vec4(
			self.x.smoothen_more(),
			self.y.smoothen_more(),
			self.z.smoothen_more(),
			self.w.smoothen_more(),
		)
	}
	fn smoothstep(self, edge0: Self, edge1: Self) -> Self {
		vec4(
			self.x.smoothstep(edge0.x, edge1.x),
			self.y.smoothstep(edge0.y, edge1.y),
			self.z.smoothstep(edge0.z, edge1.z),
			self.w.smoothstep(edge0.w, edge1.w),
		)
	}

	fn lerp_vec(self, other: Self, t: Self) -> Self {
		vec4(
			self.x.lerp(other.x, t.x),
			self.y.lerp(other.y, t.y),
			self.z.lerp(other.z, t.z),
			self.w.lerp(other.w, t.w),
		)
	}
}
