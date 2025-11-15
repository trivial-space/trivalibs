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

pub(crate) struct FormGPUBuffers {
	pub vertex_buffer: wgpu::Buffer,
	pub vertex_buffer_max_size: u64,
	pub vertex_buffer_current_size: u64,
	pub vertex_count: u32,

	pub index_buffer: Option<wgpu::Buffer>,
	pub index_buffer_max_size: u64,
	pub index_buffer_current_size: u64,
	pub index_count: u32,
}

pub(crate) struct FormStorage {
	pub buffers: Vec<FormGPUBuffers>,
	/// Number of buffers that contain valid data and should be rendered.
	/// May be less than buffers.len() if the form was resized to fewer geometries.
	/// Buffers beyond this index remain allocated but are not rendered.
	pub currently_active_buffers: usize,
	pub props: FormProps,
}

fn create_form_gpu_buffers(device: &wgpu::Device, vertex_size: u64) -> FormGPUBuffers {
	let padded_size = get_padded_size(vertex_size);
	let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
		label: None,
		usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
		size: padded_size,
		mapped_at_creation: false,
	});

	FormGPUBuffers {
		vertex_buffer,
		vertex_buffer_max_size: padded_size,
		vertex_buffer_current_size: 0,
		vertex_count: 0,
		index_buffer: None,
		index_buffer_max_size: 0,
		index_buffer_current_size: 0,
		index_count: 0,
	}
}

pub struct FormData<'a, T>
where
	T: bytemuck::Pod + bytemuck::Zeroable,
{
	pub vertex_buffer: &'a [T],
	pub index_buffer: Option<&'a [u32]>,
}

pub struct FormBuffer<'a> {
	vertex_buffer: &'a [u8],
	vertex_count: u32,
	index_buffer: Option<&'a [u8]>,
	index_count: u32,
}

impl<'a, T> Into<FormBuffer<'a>> for FormData<'a, T>
where
	T: bytemuck::Pod + bytemuck::Zeroable,
{
	fn into(self) -> FormBuffer<'a> {
		FormBuffer {
			vertex_buffer: bytemuck::cast_slice(self.vertex_buffer),
			vertex_count: self.vertex_buffer.len() as u32,
			index_buffer: self.index_buffer.map(|i| bytemuck::cast_slice(i)),
			index_count: self.index_buffer.map(|i| i.len() as u32).unwrap_or(0),
		}
	}
}

impl<'a> Into<FormBuffer<'a>> for &'a BufferedGeometry {
	fn into(self) -> FormBuffer<'a> {
		FormBuffer {
			vertex_buffer: self.vertex_buffer.as_slice(),
			vertex_count: self.vertex_count,
			index_buffer: self.index_buffer.as_deref(),
			index_count: self.index_count,
		}
	}
}

impl<'a, T: bytemuck::Pod> Into<FormBuffer<'a>> for &'a [T] {
	fn into(self) -> FormBuffer<'a> {
		FormBuffer {
			vertex_buffer: bytemuck::cast_slice(self),
			index_buffer: None,
			vertex_count: self.len() as u32,
			index_count: 0,
		}
	}
}

impl<'a, T: bytemuck::Pod> Into<FormBuffer<'a>> for &'a Vec<T> {
	fn into(self) -> FormBuffer<'a> {
		FormBuffer {
			vertex_buffer: bytemuck::cast_slice(self.as_slice()),
			index_buffer: None,
			vertex_count: self.len() as u32,
			index_count: 0,
		}
	}
}

#[derive(Clone, Copy)]
pub struct Form(pub(crate) usize);

impl Form {
	pub fn update_all<'a, I, B>(&self, painter: &mut Painter, buffers: I)
	where
		I: IntoIterator<Item = B>,
		B: Into<FormBuffer<'a>>,
	{
		let f = &mut painter.forms[self.0];
		let buffers: Vec<FormBuffer<'a>> = buffers.into_iter().map(|b| b.into()).collect();

		// Allocate new buffers if we need more than we currently have
		while f.buffers.len() < buffers.len() {
			let buf_index = f.buffers.len();
			let vertex_size = buffers[buf_index].vertex_buffer.len() as u64;
			f.buffers
				.push(create_form_gpu_buffers(&painter.device, vertex_size));
		}

		for (i, buf) in buffers.iter().enumerate() {
			let f_buf = &mut f.buffers[i];

			// Handle vertex buffer
			let vertex_size = buf.vertex_buffer.len() as u64;
			let padded_vertex_size = get_padded_size(vertex_size);

			// If the new data is larger than max_size, recreate the vertex buffer
			if f_buf.vertex_buffer_max_size < padded_vertex_size {
				f_buf.vertex_buffer = painter.device.create_buffer(&wgpu::BufferDescriptor {
					label: None,
					usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
					size: padded_vertex_size,
					mapped_at_creation: false,
				});
				f_buf.vertex_buffer_max_size = padded_vertex_size;
			}

			f_buf.vertex_count = buf.vertex_count;
			f_buf.vertex_buffer_current_size = vertex_size;

			painter
				.queue
				.write_buffer(&f_buf.vertex_buffer, 0, &buf.vertex_buffer);

			// Handle index buffer
			if let Some(index_data) = buf.index_buffer {
				let index_size = index_data.len() as u64;
				let padded_index_size = get_padded_size(index_size);

				// If buffer doesn't exist yet (lazy init) or new data is larger, (re)create it
				if f_buf.index_buffer.is_none() || f_buf.index_buffer_max_size < padded_index_size {
					f_buf.index_buffer =
						Some(painter.device.create_buffer(&wgpu::BufferDescriptor {
							label: None,
							usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
							size: padded_index_size,
							mapped_at_creation: false,
						}));
					f_buf.index_buffer_max_size = padded_index_size;
				}

				f_buf.index_count = buf.index_count;
				f_buf.index_buffer_current_size = index_size;

				let index_buffer = f_buf.index_buffer.as_ref().unwrap();
				painter.queue.write_buffer(index_buffer, 0, &index_data);
			} else {
				f_buf.index_count = 0;
				f_buf.index_buffer_current_size = 0;
			}
		}

		// Update the count of buffers that contain valid data
		f.currently_active_buffers = buffers.len();
	}

	pub fn update<'a>(&self, painter: &mut Painter, buffer: impl Into<FormBuffer<'a>>) {
		self.update_all(painter, vec![buffer.into()]);
	}

	pub fn new_with_sizes(painter: &mut Painter, sizes: &[u64], props: FormProps) -> Self {
		let mut buffers = Vec::with_capacity(sizes.len());
		for size in sizes {
			buffers.push(create_form_gpu_buffers(&painter.device, *size));
		}

		let f = FormStorage {
			buffers,
			currently_active_buffers: 0, // No data yet - will be set by update_all
			props,
		};

		let i = painter.forms.len();
		painter.forms.push(f);

		return Form(i);
	}

	pub fn new<'a>(
		painter: &mut Painter,
		buffers: impl Into<Vec<FormBuffer<'a>>>,
		props: FormProps,
	) -> Self {
		let buffers = buffers.into();
		let sizes = buffers
			.iter()
			.map(|b| b.vertex_buffer.len() as u64)
			.collect::<Vec<u64>>();
		let form = Form::new_with_sizes(painter, sizes.as_slice(), props);

		form.update_all(painter, buffers);

		form
	}
}

pub struct FormBuilder<'a, 'b> {
	painter: &'a mut Painter,
	buffers: Vec<FormBuffer<'b>>,
	sizes: Vec<u64>,
	props: FormProps,
}

impl<'a, 'b> FormBuilder<'a, 'b> {
	pub fn new(painter: &'a mut Painter) -> Self {
		FormBuilder {
			painter,
			buffers: Vec::with_capacity(1),
			sizes: Vec::with_capacity(1),
			props: FormProps::default(),
		}
	}

	pub fn create(self) -> Form {
		if self.sizes.len() == 0 {
			return Form::new(self.painter, self.buffers, self.props);
		}
		let f = Form::new_with_sizes(self.painter, &self.sizes, self.props);
		f.update_all(self.painter, self.buffers);
		f
	}

	pub fn with_sizes(mut self, sizes: Vec<u64>) -> Self {
		self.sizes = sizes;
		self
	}

	pub fn with_size(mut self, size: u64) -> Self {
		self.sizes.push(size);
		self
	}

	pub fn with_buffer(mut self, buffer: impl Into<FormBuffer<'b>>) -> Self {
		self.buffers.push(buffer.into());
		self
	}

	pub fn with_buffers<I, B>(mut self, buffers: I) -> Self
	where
		I: IntoIterator<Item = B>,
		B: Into<FormBuffer<'b>>,
	{
		self.buffers = buffers.into_iter().map(|b| b.into()).collect();
		self
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
