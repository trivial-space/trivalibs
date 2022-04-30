#[inline]
pub fn default<T: Default>() -> T {
    std::default::Default::default()
}
