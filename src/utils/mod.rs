pub mod app_state;
pub mod f32_utils;
pub mod rand_utils;

pub fn default<T: Default>() -> T {
    std::default::Default::default()
}
