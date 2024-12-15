pub mod data_structures;
pub mod geometry;
pub mod macros;
pub mod rendering;
pub mod utils;
pub mod wasm_helpers;

pub use bytemuck;
pub use glam;

pub mod prelude {
	pub use crate::geometry::interpolation::*;
	pub use crate::macros::*;
	pub use crate::utils::rand_utils::*;
	pub use crate::utils::*;
	pub use glam::*;
	pub use lerp::*;
	pub use rand::prelude::*;
}
