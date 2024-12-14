use std::f32::consts::{FRAC_PI_2, TAU};

use crate::utils::default;

use super::transform::Transform;
use glam::{vec3, Mat4, Quat, Vec2, Vec3, Vec3Swizzles, Vec4};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PerspectiveCamera {
	fov: f32,
	aspect_ratio: f32,
	near: f32,
	far: f32,

	proj: Mat4,

	pub rot_horizontal: f32,
	pub rot_vertical: f32,
	pub translation: Vec3,

	calculate_planes: bool,
	calculate_near_far_planes: bool,
}

impl Default for PerspectiveCamera {
	fn default() -> Self {
		// PerspectiveCamera::from_perspective(std::f32::consts::PI / 4.0, 1.0, 0.1, 1000.0)
		PerspectiveCamera {
			fov: 1.0,
			aspect_ratio: 1.0,
			near: 0.1,
			far: 1000.0,
			proj: Mat4::ZERO,
			rot_horizontal: 0.0,
			rot_vertical: 0.0,
			translation: Vec3::ZERO,
			calculate_planes: false,
			calculate_near_far_planes: false,
		}
	}
}

pub struct CamProps {
	pub fov: Option<f32>,
	pub aspect_ratio: Option<f32>,
	pub near: Option<f32>,
	pub far: Option<f32>,

	pub rot_horizontal: Option<f32>,
	pub rot_vertical: Option<f32>,
	pub translation: Option<Vec3>,

	pub calculate_planes: Option<bool>,
	pub calculate_near_far_planes: Option<bool>,
}

impl Default for CamProps {
	fn default() -> Self {
		Self {
			fov: None,
			aspect_ratio: None,
			near: None,
			far: None,
			rot_horizontal: None,
			rot_vertical: None,
			translation: None,
			calculate_planes: None,
			calculate_near_far_planes: None,
		}
	}
}

impl PerspectiveCamera {
	pub fn create(props: CamProps) -> Self {
		let mut cam = Self::default();
		cam.set(props);
		cam
	}

	pub fn set(&mut self, opts: CamProps) {
		let mut update_projection = false;
		if let Some(fov) = opts.fov {
			if fov != self.fov {
				self.fov = fov;
				update_projection = true;
			}
		}
		if let Some(near) = opts.near {
			if near != self.near {
				self.near = near;
				update_projection = true;
			}
		}
		if let Some(far) = opts.far {
			if far != self.far {
				self.far = far;
				update_projection = true;
			}
		}
		if let Some(ratio) = opts.aspect_ratio {
			if ratio != self.aspect_ratio {
				self.aspect_ratio = ratio;
				update_projection = true;
			}
		}
		if let Some(calculate_planes) = opts.calculate_planes {
			if calculate_planes != self.calculate_planes {
				self.calculate_planes = calculate_planes;
				update_projection = calculate_planes;
			}
		}
		if let Some(calculate_near_far_planes) = opts.calculate_near_far_planes {
			if calculate_near_far_planes != self.calculate_near_far_planes {
				self.calculate_near_far_planes = calculate_near_far_planes;
				update_projection = calculate_near_far_planes;
			}
		}
		if update_projection {
			self.recalculate_projection();
		}

		if let Some(rot_horizontal) = opts.rot_horizontal {
			self.rot_horizontal = rot_horizontal;
			if self.rot_horizontal > TAU {
				self.rot_horizontal -= TAU;
			}
			if self.rot_horizontal < 0.0 {
				self.rot_horizontal += TAU
			}
		}
		if let Some(rot_vertical) = opts.rot_vertical {
			self.rot_vertical = rot_vertical;
			if self.rot_vertical > FRAC_PI_2 {
				self.rot_vertical = FRAC_PI_2;
			}
			if self.rot_vertical < -FRAC_PI_2 {
				self.rot_vertical = -FRAC_PI_2
			}
		}
		if let Some(translation) = opts.translation {
			self.translation = translation;
		}
	}

	pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
		if self.aspect_ratio != aspect_ratio {
			self.aspect_ratio = aspect_ratio;
			self.recalculate_projection();
		}
	}

	pub fn reset_transform(&mut self, pos: Vec3, rot_horizontal: f32, rot_vertical: f32) {
		self.set(CamProps {
			rot_horizontal: Some(rot_horizontal),
			rot_vertical: Some(rot_vertical),
			translation: Some(pos),
			..default()
		})
	}

	pub fn update_transform(
		&mut self,
		forward: f32,
		left: f32,
		up: f32,
		rot_hor_delta: f32,
		rot_vert_delta: f32,
	) {
		if rot_hor_delta != 0.0 || rot_vert_delta != 0.0 {
			self.set(CamProps {
				rot_horizontal: Some(self.rot_horizontal + rot_hor_delta),
				rot_vertical: Some(self.rot_vertical + rot_vert_delta),
				..default()
			})
		}
		let mut translation = self.translation;
		if up != 0.0 {
			translation += Vec3::Y * up;
		}
		if forward != 0.0 {
			let angle = self.rot_horizontal;
			translation += vec3(-f32::sin(angle), 0.0, -f32::cos(angle)) * forward;
		}
		if left != 0.0 {
			let angle = self.rot_horizontal;
			translation += vec3(-f32::cos(angle), 0.0, f32::sin(angle)) * left;
		}
		self.translation = translation;
	}

	pub fn transform(&self) -> Transform {
		let mut t = Transform::default();
		t.translation = self.translation;
		t.rotation =
			Quat::from_rotation_y(self.rot_horizontal) * Quat::from_rotation_x(self.rot_vertical);
		t
	}

	pub fn projection_mat(&self) -> Mat4 {
		self.proj
	}

	pub fn view_mat(&self) -> Mat4 {
		self.transform().compute_matrix().inverse()
	}

	pub fn view_proj_mat(&self) -> Mat4 {
		self.projection_mat() * self.view_mat()
	}

	pub fn reflected_cam(&self, _plane: Vec4) -> PerspectiveCamera {
		todo!("reflect translation and rotations around plane")
	}

	pub fn reflected_cam_ground(&self) -> PerspectiveCamera {
		PerspectiveCamera {
			rot_vertical: -self.rot_vertical,
			translation: vec3(self.translation.x, -self.translation.y, self.translation.z),
			..*self
		}
	}

	pub fn recalculate_projection(&mut self) {
		self.proj = Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far);
	}

	/// Given a position in world space, use the camera to compute the screen space coordinates.
	///
	/// To get the coordinates in Normalized Device Coordinates, you should use
	/// [`world_to_ndc`](Self::world_to_ndc).
	pub fn world_to_screen(&self, frame_size: Vec2, world_position: Vec3) -> Option<Vec2> {
		let ndc_space_coords = self.world_to_ndc(world_position)?;
		// NDC z-values outside of 0 < z < 1 are outside the camera frustum and are thus not in screen space
		if ndc_space_coords.z < 0.0 || ndc_space_coords.z > 1.0 {
			return None;
		}

		// Once in NDC space, we can discard the z element and rescale x/y to fit the screen
		Some((ndc_space_coords.xy() + Vec2::ONE) / 2.0 * frame_size)
	}

	/// Given a position in world space, use the camera to compute the Normalized Device Coordinates.
	///
	/// Values returned will be between -1.0 and 1.0 when the position is in screen space.
	/// To get the coordinates in the render target dimensions, you should use
	/// [`world_to_screen`](Self::world_to_screen).
	pub fn world_to_ndc(&self, world_position: Vec3) -> Option<Vec3> {
		// Build a transform to convert from world to NDC using camera data
		let world_to_ndc: Mat4 = self.view_proj_mat();
		let ndc_space_coords: Vec3 = world_to_ndc.project_point3(world_position);

		if !ndc_space_coords.is_nan() {
			Some(ndc_space_coords)
		} else {
			None
		}
	}

	// TODO: Implement screen_to_world_ray and ndc_to_world_ray
}
