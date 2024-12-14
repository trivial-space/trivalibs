use glam::{Affine3A, Mat3, Mat4, Quat, Vec3};
use serde::Serialize;
use std::ops::Mul;

/// Describe the position of an entity.
/// To place or move an entity, you should set its [`Transform`].
/// Copied from Bevy Engine Transform (version 0.10)
#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub struct Transform {
	/// Position of the entity. In 2d, the last value of the `Vec3` is used for z-ordering.
	pub translation: Vec3,
	/// Rotation of the entity.
	pub rotation: Quat,
	/// Scale of the entity.
	pub scale: Vec3,
}

impl Transform {
	/// An identity [`Transform`] with no translation, rotation, and a scale of 1 on all axes.
	pub const IDENTITY: Self = Transform {
		translation: Vec3::ZERO,
		rotation: Quat::IDENTITY,
		scale: Vec3::ONE,
	};

	/// Creates a new [`Transform`] at the position `(x, y, z)`. In 2d, the `z` component
	/// is used for z-ordering elements: higher `z`-value will be in front of lower
	/// `z`-value.
	#[inline]
	pub const fn from_xyz(x: f32, y: f32, z: f32) -> Self {
		Self::from_translation(Vec3::new(x, y, z))
	}

	/// Extracts the translation, rotation, and scale from `matrix`. It must be a 3d affine
	/// transformation matrix.
	#[inline]
	pub fn from_matrix(matrix: Mat4) -> Self {
		let (scale, rotation, translation) = matrix.to_scale_rotation_translation();

		Transform {
			translation,
			rotation,
			scale,
		}
	}

	/// Creates a new [`Transform`], with `translation`. Rotation will be 0 and scale 1 on
	/// all axes.
	#[inline]
	pub const fn from_translation(translation: Vec3) -> Self {
		Transform {
			translation,
			..Self::IDENTITY
		}
	}

	/// Creates a new [`Transform`], with `rotation`. Translation will be 0 and scale 1 on
	/// all axes.
	#[inline]
	pub const fn from_rotation(rotation: Quat) -> Self {
		Transform {
			rotation,
			..Self::IDENTITY
		}
	}

	/// Creates a new [`Transform`], with `scale`. Translation will be 0 and rotation 0 on
	/// all axes.
	#[inline]
	pub const fn from_scale(scale: Vec3) -> Self {
		Transform {
			scale,
			..Self::IDENTITY
		}
	}

	/// Returns this [`Transform`] with a new rotation so that [`Transform::forward`]
	/// points towards the `target` position and [`Transform::up`] points towards `up`.
	///
	/// In some cases it's not possible to construct a rotation. Another axis will be picked in those cases:
	/// * if `target` is the same as the transform translation, `Vec3::Z` is used instead
	/// * if `up` is zero, `Vec3::Y` is used instead
	/// * if the resulting forward direction is parallel with `up`, an orthogonal vector is used as the "right" direction
	#[inline]
	#[must_use]
	pub fn looking_at(mut self, target: Vec3, up: Vec3) -> Self {
		self.look_at(target, up);
		self
	}

	/// Returns this [`Transform`] with a new rotation so that [`Transform::forward`]
	/// points in the given `direction` and [`Transform::up`] points towards `up`.
	///
	/// In some cases it's not possible to construct a rotation. Another axis will be picked in those cases:
	/// * if `direction` is zero, `Vec3::Z` is used instead
	/// * if `up` is zero, `Vec3::Y` is used instead
	/// * if `direction` is parallel with `up`, an orthogonal vector is used as the "right" direction
	#[inline]
	#[must_use]
	pub fn looking_to(mut self, direction: Vec3, up: Vec3) -> Self {
		self.look_to(direction, up);
		self
	}

	/// Returns this [`Transform`] with a new translation.
	#[inline]
	#[must_use]
	pub const fn with_translation(mut self, translation: Vec3) -> Self {
		self.translation = translation;
		self
	}

	/// Returns this [`Transform`] with a new rotation.
	#[inline]
	#[must_use]
	pub const fn with_rotation(mut self, rotation: Quat) -> Self {
		self.rotation = rotation;
		self
	}

	/// Returns this [`Transform`] with a new scale.
	#[inline]
	#[must_use]
	pub const fn with_scale(mut self, scale: Vec3) -> Self {
		self.scale = scale;
		self
	}

	/// Returns the 3d affine transformation matrix from this transforms translation,
	/// rotation, and scale.
	#[inline]
	pub fn compute_matrix(&self) -> Mat4 {
		Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
	}

	/// Returns the 3d affine transformation matrix from this transforms translation,
	/// rotation, and scale.
	#[inline]
	pub fn compute_affine(&self) -> Affine3A {
		Affine3A::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
	}

	/// Get the unit vector in the local `X` direction.
	#[inline]
	pub fn local_x(&self) -> Vec3 {
		self.rotation * Vec3::X
	}

	/// Equivalent to [`-local_x()`][Transform::local_x()]
	#[inline]
	pub fn left(&self) -> Vec3 {
		-self.local_x()
	}

	/// Equivalent to [`local_x()`][Transform::local_x()]
	#[inline]
	pub fn right(&self) -> Vec3 {
		self.local_x()
	}

	/// Get the unit vector in the local `Y` direction.
	#[inline]
	pub fn local_y(&self) -> Vec3 {
		self.rotation * Vec3::Y
	}

	/// Equivalent to [`local_y()`][Transform::local_y]
	#[inline]
	pub fn up(&self) -> Vec3 {
		self.local_y()
	}

	/// Equivalent to [`-local_y()`][Transform::local_y]
	#[inline]
	pub fn down(&self) -> Vec3 {
		-self.local_y()
	}

	/// Get the unit vector in the local `Z` direction.
	#[inline]
	pub fn local_z(&self) -> Vec3 {
		self.rotation * Vec3::Z
	}

	/// Equivalent to [`-local_z()`][Transform::local_z]
	#[inline]
	pub fn forward(&self) -> Vec3 {
		-self.local_z()
	}

	/// Equivalent to [`local_z()`][Transform::local_z]
	#[inline]
	pub fn back(&self) -> Vec3 {
		self.local_z()
	}

	/// Rotates this [`Transform`] by the given rotation.
	#[inline]
	pub fn rotate(&mut self, rotation: Quat) {
		self.rotation = rotation * self.rotation;
	}

	/// Rotates this [`Transform`] around the given `axis` by `angle` (in radians).
	#[inline]
	pub fn rotate_axis(&mut self, axis: Vec3, angle: f32) {
		self.rotate(Quat::from_axis_angle(axis, angle));
	}

	/// Rotates this [`Transform`] around the `X` axis by `angle` (in radians).
	#[inline]
	pub fn rotate_x(&mut self, angle: f32) {
		self.rotate(Quat::from_rotation_x(angle));
	}

	/// Rotates this [`Transform`] around the `Y` axis by `angle` (in radians).
	#[inline]
	pub fn rotate_y(&mut self, angle: f32) {
		self.rotate(Quat::from_rotation_y(angle));
	}

	/// Rotates this [`Transform`] around the `Z` axis by `angle` (in radians).
	#[inline]
	pub fn rotate_z(&mut self, angle: f32) {
		self.rotate(Quat::from_rotation_z(angle));
	}

	/// Rotates this [`Transform`] by the given `rotation`.
	///
	/// The `rotation` is relative to this [`Transform`]'s current rotation.
	#[inline]
	pub fn rotate_local(&mut self, rotation: Quat) {
		self.rotation *= rotation;
	}

	/// Rotates this [`Transform`] around its local `axis` by `angle` (in radians).
	#[inline]
	pub fn rotate_local_axis(&mut self, axis: Vec3, angle: f32) {
		self.rotate_local(Quat::from_axis_angle(axis, angle));
	}

	/// Rotates this [`Transform`] around its local `X` axis by `angle` (in radians).
	#[inline]
	pub fn rotate_local_x(&mut self, angle: f32) {
		self.rotate_local(Quat::from_rotation_x(angle));
	}

	/// Rotates this [`Transform`] around its local `Y` axis by `angle` (in radians).
	#[inline]
	pub fn rotate_local_y(&mut self, angle: f32) {
		self.rotate_local(Quat::from_rotation_y(angle));
	}

	/// Rotates this [`Transform`] around its local `Z` axis by `angle` (in radians).
	#[inline]
	pub fn rotate_local_z(&mut self, angle: f32) {
		self.rotate_local(Quat::from_rotation_z(angle));
	}

	/// Translates this [`Transform`] around a `point` in space.
	#[inline]
	pub fn translate_around(&mut self, point: Vec3, rotation: Quat) {
		self.translation = point + rotation * (self.translation - point);
	}

	/// Rotates this [`Transform`] around a `point` in space.
	#[inline]
	pub fn rotate_around(&mut self, point: Vec3, rotation: Quat) {
		self.translate_around(point, rotation);
		self.rotate(rotation);
	}

	/// Rotates this [`Transform`] so that [`Transform::forward`] points towards the `target` position,
	/// and [`Transform::up`] points towards `up`.
	///
	/// In some cases it's not possible to construct a rotation. Another axis will be picked in those cases:
	/// * if `target` is the same as the transtorm translation, `Vec3::Z` is used instead
	/// * if `up` is zero, `Vec3::Y` is used instead
	/// * if the resulting forward direction is parallel with `up`, an orthogonal vector is used as the "right" direction
	#[inline]
	pub fn look_at(&mut self, target: Vec3, up: Vec3) {
		self.look_to(target - self.translation, up);
	}

	/// Rotates this [`Transform`] so that [`Transform::forward`] points in the given `direction`
	/// and [`Transform::up`] points towards `up`.
	///
	/// In some cases it's not possible to construct a rotation. Another axis will be picked in those cases:
	/// * if `direction` is zero, `Vec3::NEG_Z` is used instead
	/// * if `up` is zero, `Vec3::Y` is used instead
	/// * if `direction` is parallel with `up`, an orthogonal vector is used as the "right" direction
	#[inline]
	pub fn look_to(&mut self, direction: Vec3, up: Vec3) {
		let back = -direction.try_normalize().unwrap_or(Vec3::NEG_Z);
		let up = up.try_normalize().unwrap_or(Vec3::Y);
		let right = up
			.cross(back)
			.try_normalize()
			.unwrap_or_else(|| up.any_orthonormal_vector());
		let up = back.cross(right);
		self.rotation = Quat::from_mat3(&Mat3::from_cols(right, up, back));
	}

	/// Multiplies `self` with `transform` component by component, returning the
	/// resulting [`Transform`]
	#[inline]
	#[must_use]
	pub fn mul_transform(&self, transform: Transform) -> Self {
		let translation = self.transform_point(transform.translation);
		let rotation = self.rotation * transform.rotation;
		let scale = self.scale * transform.scale;
		Transform {
			translation,
			rotation,
			scale,
		}
	}

	/// Transforms the given `point`, applying scale, rotation and translation.
	#[inline]
	pub fn transform_point(&self, mut point: Vec3) -> Vec3 {
		point = self.scale * point;
		point = self.rotation * point;
		point += self.translation;
		point
	}
}

impl Default for Transform {
	fn default() -> Self {
		Self::IDENTITY
	}
}

impl Mul<Transform> for Transform {
	type Output = Transform;

	fn mul(self, transform: Transform) -> Self::Output {
		self.mul_transform(transform)
	}
}

impl Mul<Vec3> for Transform {
	type Output = Vec3;

	fn mul(self, value: Vec3) -> Self::Output {
		self.transform_point(value)
	}
}
