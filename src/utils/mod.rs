pub mod app_state;

pub fn default<T: Default>() -> T {
    std::default::Default::default()
}
