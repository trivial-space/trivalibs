pub mod app_state;
pub mod rand_utils;

pub fn default<T: Default>() -> T {
	std::default::Default::default()
}
