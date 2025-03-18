use crate::{layer::Layer, painter::get_padded_size, sampler::Sampler, texture::Texture, Painter};
use trivalibs_core::glam::{Mat3, Mat3A, Vec3, Vec3A};
use wgpu::{BindingType, ShaderStages};

#[derive(Clone, Copy)]
pub struct UniformLayout {
	pub(crate) binding_type: BindingType,
	pub(crate) visibility: ShaderStages,
}

#[derive(Clone, Copy)]
pub struct LayerLayout {
	pub(crate) visibility: ShaderStages,
}

#[derive(Clone, Copy)]
pub enum Uniform {
	Buffer(Buffer),
	Tex2D(Texture),
	Sampler(Sampler),
}

#[derive(Clone, Copy)]
pub struct Buffer(pub(crate) usize);

impl Buffer {
	pub fn uniform(&self) -> Uniform {
		Uniform::Buffer(*self)
	}
}

pub struct UniformBuffer<T> {
	buffer: Buffer,
	t: std::marker::PhantomData<T>,
}

impl<T> UniformBuffer<T>
where
	T: bytemuck::Pod,
{
	pub fn new(painter: &mut Painter, data: T) -> Self {
		let buffer = painter.device.create_buffer(&wgpu::BufferDescriptor {
			label: None,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			size: get_padded_size(std::mem::size_of::<T>() as u64),
			mapped_at_creation: false,
		});

		painter.buffers.push(buffer);

		let buffer = Buffer(painter.buffers.len() - 1);

		let uniform = UniformBuffer {
			buffer,
			t: std::marker::PhantomData,
		};

		uniform.update(&painter, data);

		uniform
	}

	pub fn update(&self, painter: &Painter, data: T) {
		let buffer = &painter.buffers[self.buffer.0];
		painter
			.queue
			.write_buffer(buffer, 0, bytemuck::cast_slice(&[data]));
	}

	pub fn uniform(&self) -> Uniform {
		Uniform::Buffer(self.buffer)
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable)]
pub struct Mat3U(pub(crate) Mat3A);
unsafe impl bytemuck::Pod for Mat3U {}

impl UniformBuffer<Mat3U> {
	pub fn new_mat3(painter: &mut Painter, data: Mat3) -> Self {
		UniformBuffer::new(painter, Mat3U(Mat3A::from(data)))
	}

	pub fn update_mat3(&self, painter: &Painter, data: Mat3) {
		self.update(painter, Mat3U(Mat3A::from(data)));
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable)]
pub struct Vec3U(pub(crate) Vec3A);
unsafe impl bytemuck::Pod for Vec3U {}

impl UniformBuffer<Vec3U> {
	pub fn new_vec3(painter: &mut Painter, data: Vec3) -> Self {
		UniformBuffer::new(painter, Vec3U(Vec3A::from(data)))
	}

	pub fn update_vec3(&self, painter: &Painter, data: Vec3) {
		self.update(painter, Vec3U(Vec3A::from(data)));
	}
}

#[derive(Clone)]
pub struct InstanceUniforms {
	pub uniforms: Vec<(u32, Uniform)>,
	pub effect_layers: Vec<(u32, Layer)>,
}

impl Default for InstanceUniforms {
	fn default() -> Self {
		Self {
			uniforms: Vec::with_capacity(0),
			effect_layers: Vec::with_capacity(0),
		}
	}
}
