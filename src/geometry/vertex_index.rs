use std::hash::Hash;

use glam::Vec3;

pub trait WithVertexIndex<T: VertexIndex> {
    fn vertex_index(&self) -> T;
}

pub trait VertexIndex: Eq + Hash {}

#[derive(PartialEq, Eq)]
pub struct VertIdx2U(u32, u32);
impl VertexIndex for VertIdx2U {}
impl Hash for VertIdx2U {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash_val: u64 = self.0 as u64 + self.1 as u64 * 100_000_000;
        hash_val.hash(state)
    }
}

#[derive(PartialEq, Eq)]
pub struct VertIdx2Usize(usize, usize);
impl VertexIndex for VertIdx2Usize {}
impl Hash for VertIdx2Usize {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash_val: u64 = self.0 as u64 + self.1 as u64 * 100_000_000;
        hash_val.hash(state)
    }
}

#[derive(PartialEq)]
pub struct VertIdx3f(f32, f32, f32);
impl VertexIndex for VertIdx3f {}

impl From<Vec3> for VertIdx3f {
    fn from(v: Vec3) -> Self {
        Self(v.x, v.y, v.z)
    }
}
impl Eq for VertIdx3f {}
impl Hash for VertIdx3f {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash_val: f64 =
            self.0 as f64 + self.1 as f64 * 1_000_000_f64 + self.2 as f64 * 1_000_000_000_000_f64;
        hash_val.to_ne_bytes().hash(state)
    }
}
