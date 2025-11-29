#![no_std]
#![allow(unexpected_cfgs)]

pub mod bits;
pub mod blur;
pub mod color;
pub mod coords;
pub mod num_ext;
pub mod random;
pub mod vec_ext;

pub mod prelude {
	pub use crate::bits::*;
	pub use crate::num_ext::*;
	pub use crate::random::*;
	pub use crate::vec_ext::*;

	#[cfg(target_arch = "spirv")]
	pub use spirv_std::num_traits::*;
}
