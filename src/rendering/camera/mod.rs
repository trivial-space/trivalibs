use super::transform::Transform;
use glam::{Mat4, Vec2, Vec3};

#[derive(Debug, Clone)]
pub struct PerspectiveCamera {
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,

    pub proj: Mat4,
    pub transform: Transform,
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        PerspectiveCamera::from_perspective(std::f32::consts::PI / 4.0, 1.0, 0.1, 1000.0)
    }
}

impl PerspectiveCamera {
    pub fn from_perspective(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        PerspectiveCamera {
            fov,
            aspect_ratio,
            near,
            far,
            proj: Mat4::ZERO,
            transform: Default::default(),
        }
    }

    pub fn update_projection_matrix(&mut self) {
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
        Some((ndc_space_coords.truncate() + Vec2::ONE) / 2.0 * frame_size)
    }

    /// Given a position in world space, use the camera to compute the Normalized Device Coordinates.
    ///
    /// Values returned will be between -1.0 and 1.0 when the position is in screen space.
    /// To get the coordinates in the render target dimensions, you should use
    /// [`world_to_screen`](Self::world_to_screen).
    pub fn world_to_ndc(&self, world_position: Vec3) -> Option<Vec3> {
        // Build a transform to convert from world to NDC using camera data
        let world_to_ndc: Mat4 = self.proj * self.transform.compute_matrix().inverse();
        let ndc_space_coords: Vec3 = world_to_ndc.project_point3(world_position);

        if !ndc_space_coords.is_nan() {
            Some(ndc_space_coords)
        } else {
            None
        }
    }
}
