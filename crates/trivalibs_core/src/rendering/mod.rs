pub mod camera;
pub mod line_2d;
pub mod mesh_geometry;
pub mod mesh_geometry_2;
pub mod scene;
pub mod shapes;
pub mod texture;
pub mod webgl_buffered_geometry;

pub struct BufferedGeometry {
	pub vertex_buffer: Vec<u8>,
	pub index_buffer: Option<Vec<u8>>,
	pub vertex_count: u32,
	pub index_count: u32,
}

impl<T: bytemuck::Pod> Into<BufferedGeometry> for &[T] {
	fn into(self) -> BufferedGeometry {
		BufferedGeometry {
			vertex_buffer: bytemuck::cast_slice(self).to_vec(),
			index_buffer: None,
			vertex_count: self.len() as u32,
			index_count: 0,
		}
	}
}

impl<T: bytemuck::Pod> Into<BufferedGeometry> for Vec<T> {
	fn into(self) -> BufferedGeometry {
		BufferedGeometry {
			vertex_buffer: bytemuck::cast_slice(&self).to_vec(),
			index_buffer: None,
			vertex_count: self.len() as u32,
			index_count: 0,
		}
	}
}
