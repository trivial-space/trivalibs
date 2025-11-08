use spirv_std::glam::{UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};

pub trait FloatBits<T> {
	fn to_bits(self) -> T;
	fn from_bits(v: T) -> Self;
}

pub trait FromBits<T> {}

impl FloatBits<UVec2> for Vec2 {
	fn to_bits(self) -> UVec2 {
		UVec2::new(self.x.to_bits(), self.y.to_bits())
	}
	fn from_bits(v: UVec2) -> Self {
		Vec2::new(f32::from_bits(v.x), f32::from_bits(v.y))
	}
}

impl FloatBits<UVec3> for Vec3 {
	fn to_bits(self) -> UVec3 {
		UVec3::new(self.x.to_bits(), self.y.to_bits(), self.z.to_bits())
	}
	fn from_bits(v: UVec3) -> Self {
		Vec3::new(
			f32::from_bits(v.x),
			f32::from_bits(v.y),
			f32::from_bits(v.z),
		)
	}
}

impl FloatBits<UVec4> for Vec4 {
	fn to_bits(self) -> UVec4 {
		UVec4::new(
			self.x.to_bits(),
			self.y.to_bits(),
			self.z.to_bits(),
			self.w.to_bits(),
		)
	}
	fn from_bits(v: UVec4) -> Vec4 {
		Vec4::new(
			f32::from_bits(v.x),
			f32::from_bits(v.y),
			f32::from_bits(v.z),
			f32::from_bits(v.w),
		)
	}
}
