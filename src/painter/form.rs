use crate::rendering::RenderableBuffer;

use super::{painter::get_padded_size, Painter};

pub(crate) struct FormStorage {
	pub vertex_buffer: wgpu::Buffer,
	pub index_buffer: Option<wgpu::Buffer>,
	pub vertex_count: u32,
	pub index_count: u32,
}

pub struct FormProps<'a, T>
where
	T: bytemuck::Pod + bytemuck::Zeroable,
{
	pub vertex_buffer: &'a [T],
	pub index_buffer: Option<&'a [u32]>,
}

#[derive(Clone, Copy)]
pub struct Form(pub(crate) usize);

impl Form {
	pub fn update_form<T>(&self, painter: &mut Painter, props: &FormProps<T>)
	where
		T: bytemuck::Pod + bytemuck::Zeroable,
	{
		let f = &mut painter.forms[self.0];

		f.vertex_count = props.vertex_buffer.len() as u32;

		painter.queue.write_buffer(
			&f.vertex_buffer,
			0,
			bytemuck::cast_slice(props.vertex_buffer),
		);

		if let Some(index_data) = props.index_buffer {
			f.index_count = index_data.len() as u32;

			let index_buffer = f.index_buffer.get_or_insert(painter.device.create_buffer(
				&wgpu::BufferDescriptor {
					label: None,
					usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
					size: get_padded_size(
						index_data.len() as u64 * std::mem::size_of::<u32>() as u64,
					),
					mapped_at_creation: false,
				},
			));

			painter.queue.write_buffer(
				index_buffer,
				0,
				bytemuck::cast_slice(props.index_buffer.unwrap()),
			);
		}
	}

	pub fn update_form_buffer(&self, painter: &mut Painter, buffers: RenderableBuffer) {
		let f = &mut painter.forms[self.0];

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

	pub fn new_with_size(painter: &mut Painter, size: u64) -> Self {
		let vertex_buffer = painter.device.create_buffer(&wgpu::BufferDescriptor {
			label: None,
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			size: get_padded_size(size),
			mapped_at_creation: false,
		});

		let f = FormStorage {
			vertex_buffer,
			index_buffer: None,
			index_count: 0,
			vertex_count: 0,
		};

		let i = painter.forms.len();
		painter.forms.push(f);

		return Form(i);
	}

	pub fn new<T>(painter: &mut Painter, props: &FormProps<T>) -> Self
	where
		T: bytemuck::Pod,
	{
		let form = painter.create_form_with_size(
			props.vertex_buffer.len() as u64 * std::mem::size_of::<T>() as u64,
		);

		painter.update_form(&form, props);

		form
	}
}
