use crate::{Painter, painter::get_padded_size};
use trivalibs_core::rendering::BufferedGeometry;

#[derive(Clone, Copy)]
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

pub struct FormBuffers<'a> {
	vertex_buffer: &'a [u8],
	vertex_count: u32,
	index_buffer: Option<&'a [u8]>,
	index_count: u32,
}

impl<'a, T> Into<FormBuffers<'a>> for FormData<'a, T>
where
	T: bytemuck::Pod + bytemuck::Zeroable,
{
	fn into(self) -> FormBuffers<'a> {
		FormBuffers {
			vertex_buffer: bytemuck::cast_slice(self.vertex_buffer),
			vertex_count: self.vertex_buffer.len() as u32,
			index_buffer: self.index_buffer.map(|i| bytemuck::cast_slice(i)),
			index_count: self.index_buffer.map(|i| i.len() as u32).unwrap_or(0),
		}
	}
}

impl<'a> Into<FormBuffers<'a>> for &'a BufferedGeometry {
	fn into(self) -> FormBuffers<'a> {
		FormBuffers {
			vertex_buffer: self.vertex_buffer.as_slice(),
			vertex_count: self.vertex_count,
			index_buffer: self.index_buffer.as_deref(),
			index_count: self.index_count,
		}
	}
}

impl<'a, T: bytemuck::Pod> Into<FormBuffers<'a>> for &'a [T] {
	fn into(self) -> FormBuffers<'a> {
		FormBuffers {
			vertex_buffer: bytemuck::cast_slice(self),
			index_buffer: None,
			vertex_count: self.len() as u32,
			index_count: 0,
		}
	}
}

#[derive(Clone, Copy)]
pub struct Form(pub(crate) usize);

impl Form {
	pub fn update<'a>(&self, painter: &mut Painter, buffers: &'a FormBuffers<'a>) {
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

	pub fn new<'a>(painter: &mut Painter, buffer: &'a FormBuffers<'a>, props: FormProps) -> Self {
		let form = Form::new_with_size(painter, buffer.vertex_buffer.len() as u64, props);

		form.update(painter, buffer);

		form
	}
}

pub struct FormBuilder<'a, 'b> {
	painter: &'a mut Painter,
	buffer: FormBuffers<'b>,
	props: FormProps,
}

impl<'a, 'b> FormBuilder<'a, 'b> {
	pub fn new(painter: &'a mut Painter, buffer: impl Into<FormBuffers<'b>>) -> Self {
		FormBuilder {
			buffer: buffer.into(),
			painter,
			props: FormProps::default(),
		}
	}

	pub fn create(self) -> Form {
		Form::new(self.painter, &self.buffer, self.props)
	}

	pub fn with_topology(mut self, topology: wgpu::PrimitiveTopology) -> Self {
		self.props.topology = topology;
		self
	}

	pub fn with_front_face(mut self, front_face: wgpu::FrontFace) -> Self {
		self.props.front_face = front_face;
		self
	}
}
