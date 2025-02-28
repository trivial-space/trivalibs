pub mod data;
pub mod macros;
pub mod math;
pub mod rendering;
pub mod utils;
pub mod wasm_helpers;

pub use bytemuck;
pub use glam;

pub mod prelude {
	pub use crate::data::*;
	pub use crate::macros::*;
	pub use crate::math::interpolation::*;
	pub use crate::utils::rand_utils::*;
	pub use crate::utils::*;
	pub use glam::*;
	pub use lerp::*;
	pub use rand::prelude::*;
	pub use rand::random;
}
