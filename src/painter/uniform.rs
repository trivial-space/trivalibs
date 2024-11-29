use super::{painter::get_padded_size, Painter};

pub struct Uniform<T> {
	buffer: wgpu::Buffer,
	binding: wgpu::BindGroup,
	t: std::marker::PhantomData<T>,
}

impl<'a, T> Into<Option<&'a wgpu::BindGroup>> for &'a Uniform<T> {
	fn into(self) -> Option<&'a wgpu::BindGroup> {
		Some(&self.binding)
	}
}

pub fn get_uniform_layout_buffered(
	painter: &Painter,
	visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayout {
	painter
		.device
		.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}],
			label: None,
		})
}

impl<T> Uniform<T>
where
	T: bytemuck::Pod,
{
	pub fn new_buffered(painter: &Painter, layout: &wgpu::BindGroupLayout, data: T) -> Self {
		let buffer = painter.device.create_buffer(&wgpu::BufferDescriptor {
			label: None,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			size: get_padded_size(std::mem::size_of::<T>() as u64),
			mapped_at_creation: false,
		});

		let bind_group = painter
			.device
			.create_bind_group(&wgpu::BindGroupDescriptor {
				layout,
				entries: &[wgpu::BindGroupEntry {
					binding: 0,
					resource: buffer.as_entire_binding(),
				}],
				label: None,
			});

		let uniform = Uniform {
			buffer,
			binding: bind_group,
			t: std::marker::PhantomData,
		};

		uniform.update_buffered(&painter, data);

		uniform
	}

	pub fn update_buffered(&self, painter: &Painter, data: T) {
		painter
			.queue
			.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
	}
}
