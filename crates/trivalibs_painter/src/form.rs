use crate::{painter::get_padded_size, Painter};
use trivalibs_core::rendering::RenderableBuffer;

pub struct FormProps {
	pub topology: wgpu::PrimitiveTopology,
	pub front_face: wgpu::FrontFace,
}

impl Default for FormProps {
	fn default() -> Self {
		FormProps {
			topology: wgpu::PrimitiveTopology::TriangleList,
			front_face: wgpu::FrontFace::Ccw,
		}
	}
}

pub(crate) struct FormStorage {
	pub vertex_buffer: wgpu::Buffer,
	pub index_buffer: Option<wgpu::Buffer>,
	pub vertex_count: u32,
	pub index_count: u32,
	pub props: FormProps,
}

pub struct FormData<'a, T>
where
	T: bytemuck::Pod + bytemuck::Zeroable,
{
	pub vertex_buffer: &'a [T],
	pub index_buffer: Option<&'a [u32]>,
}

impl<'a, T> Into<RenderableBuffer> for FormData<'a, T>
where
	T: bytemuck::Pod + bytemuck::Zeroable,
{
	fn into(self) -> RenderableBuffer {
		RenderableBuffer {
			vertex_buffer: bytemuck::cast_slice(self.vertex_buffer).to_vec(),
			vertex_count: self.vertex_buffer.len() as u32,
			index_buffer: self.index_buffer.map(|i| bytemuck::cast_slice(i).to_vec()),
			index_count: self.index_buffer.map(|i| i.len() as u32).unwrap_or(0),
		}
	}
}

#[derive(Clone, Copy)]
pub struct Form(pub(crate) usize);

impl Form {
	pub fn update(&self, painter: &mut Painter, buffers: impl Into<RenderableBuffer>) {
		let f = &mut painter.forms[self.0];
		let buffers = buffers.into();

		f.vertex_count = buffers.vertex_count;

		painter
			.queue
			.write_buffer(&f.vertex_buffer, 0, &buffers.vertex_buffer);

		if let Some(index_data) = buffers.index_buffer {
			f.index_count = buffers.index_count;

			let index_buffer = f.index_buffer.get_or_insert(painter.device.create_buffer(
				&wgpu::BufferDescriptor {
					label: None,
					usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
					size: get_padded_size(
						buffers.index_count as u64 * std::mem::size_of::<u32>() as u64,
					),
					mapped_at_creation: false,
				},
			));

			painter.queue.write_buffer(index_buffer, 0, &index_data);
		}
	}

	pub fn new_with_size(painter: &mut Painter, size: u64, props: FormProps) -> Self {
		let vertex_buffer = painter.device.create_buffer(&wgpu::BufferDescriptor {
			label: None,
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			size: get_padded_size(size),
			mapped_at_creation: false,
		});

		let f = FormStorage {
			vertex_buffer,
			vertex_count: 0,
			index_buffer: None,
			index_count: 0,
			props,
		};

		let i = painter.forms.len();
		painter.forms.push(f);

		return Form(i);
	}

	pub fn new(
		painter: &mut Painter,
		buffer: impl Into<RenderableBuffer>,
		props: FormProps,
	) -> Self {
		let buffer = buffer.into();
		let form = Form::new_with_size(painter, buffer.vertex_buffer.len() as u64, props);

		form.update(painter, buffer);

		form
	}
}
