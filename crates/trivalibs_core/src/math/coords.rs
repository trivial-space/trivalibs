use glam::Vec3;

/// Converts cartesian coordinates to spherical angles.
///
/// # Arguments
///
/// * `horizontal_angle` - The horizontal angle in range of 0 to 2 PI. Turns counter clock wise. Starts at -Z axis.
/// * `vertical_angle` - The vertical angle in range of -PI/2 to PI/2.
pub fn angles_to_cartesian(horizontal_angle: f32, vertical_angle: f32) -> Vec3 {
	let x = vertical_angle.cos() * horizontal_angle.sin();
	let y = vertical_angle.sin();
	let z = vertical_angle.cos() * horizontal_angle.cos();
	Vec3::new(x, y, z)
}
