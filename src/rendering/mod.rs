pub mod buffered_geometry;
pub mod camera;
pub mod objects;
pub mod scene;
pub mod texture;
pub mod transform;

pub struct RenderableBuffer {
	pub vertex_buffer: Vec<u8>,
	pub index_buffer: Option<Vec<u8>>,
	pub vertex_count: u32,
	pub index_count: u32,
}
