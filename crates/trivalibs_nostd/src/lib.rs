#![no_std]

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
}
