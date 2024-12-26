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

impl<T: bytemuck::Pod> Into<RenderableBuffer> for &[T] {
	fn into(self) -> RenderableBuffer {
		RenderableBuffer {
			vertex_buffer: bytemuck::cast_slice(self).to_vec(),
			index_buffer: None,
			vertex_count: self.len() as u32,
			index_count: 0,
		}
	}
}

impl<T: bytemuck::Pod> Into<RenderableBuffer> for Vec<T> {
	fn into(self) -> RenderableBuffer {
		RenderableBuffer {
			vertex_buffer: bytemuck::cast_slice(&self).to_vec(),
			index_buffer: None,
			vertex_count: self.len() as u32,
			index_count: 0,
		}
	}
}
