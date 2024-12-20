use super::{painter::get_padded_size, Painter};
use crate::texture::{Texture, UniformTex2D};
use trivalibs_core::glam::{Mat3, Mat3A, Mat4, UVec2, Vec2, Vec3, Vec3A, Vec4};

#[derive(Debug, Clone, Copy)]
pub struct Uniform(pub(crate) usize);

pub struct UniformBuffer<T> {
	pub uniform: Uniform,
	buffer: wgpu::Buffer,
	t: std::marker::PhantomData<T>,
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

impl<T> UniformBuffer<T>
where
	T: bytemuck::Pod,
{
	pub fn new(painter: &mut Painter, layout: &wgpu::BindGroupLayout, data: T) -> Self {
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

		painter.bindings.push(bind_group);

		let binding = Uniform(painter.bindings.len() - 1);

		let uniform = UniformBuffer {
			buffer,
			uniform: binding,
			t: std::marker::PhantomData,
		};

		uniform.update(&painter, data);

		uniform
	}

	pub fn update(&self, painter: &Painter, data: T) {
		painter
			.queue
			.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable)]
pub struct Mat3U(Mat3A);
unsafe impl bytemuck::Pod for Mat3U {}

impl UniformBuffer<Mat3U> {
	pub fn new_mat3(painter: &mut Painter, layout: &wgpu::BindGroupLayout, data: Mat3) -> Self {
		UniformBuffer::new(painter, layout, Mat3U(Mat3A::from(data)))
	}

	pub fn update_mat3(&self, painter: &Painter, data: Mat3) {
		self.update(painter, Mat3U(Mat3A::from(data)));
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable)]
pub struct Vec3U(Vec3A);
unsafe impl bytemuck::Pod for Vec3U {}

impl UniformBuffer<Vec3U> {
	pub fn new_vec3(painter: &mut Painter, layout: &wgpu::BindGroupLayout, data: Vec3) -> Self {
		UniformBuffer::new(painter, layout, Vec3U(Vec3A::from(data)))
	}

	pub fn update_vec3(&self, painter: &Painter, data: Vec3) {
		self.update(painter, Vec3U(Vec3A::from(data)));
	}
}

pub trait UniformType {
	fn create_buff<T: bytemuck::Pod>(&self, painter: &mut Painter, data: T) -> UniformBuffer<T>;
	fn create_tex2d(
		&self,
		painter: &mut Painter,
		texture: Texture,
		sampler: &wgpu::Sampler,
	) -> UniformTex2D;
	fn layout(&self) -> &wgpu::BindGroupLayout;

	fn create_mat3(&self, painter: &mut Painter) -> UniformBuffer<Mat3U> {
		self.create_buff(painter, Mat3U(Mat3A::IDENTITY))
	}
	fn create_mat4(&self, painter: &mut Painter) -> UniformBuffer<Mat4> {
		self.create_buff(painter, Mat4::IDENTITY)
	}
	fn create_vec2(&self, painter: &mut Painter) -> UniformBuffer<Vec2> {
		self.create_buff(painter, Vec2::ZERO)
	}
	fn create_vec3(&self, painter: &mut Painter) -> UniformBuffer<Vec3U> {
		self.create_buff(painter, Vec3U(Vec3A::ZERO))
	}
	fn create_vec4(&self, painter: &mut Painter) -> UniformBuffer<Vec4> {
		self.create_buff(painter, Vec4::ZERO)
	}
	fn create_uvec2(&self, painter: &mut Painter) -> UniformBuffer<UVec2> {
		self.create_buff(painter, UVec2::ZERO)
	}
	fn create_f32(&self, painter: &mut Painter) -> UniformBuffer<f32> {
		self.create_buff(painter, 0.0f32)
	}
	fn create_u32(&self, painter: &mut Painter) -> UniformBuffer<u32> {
		self.create_buff(painter, 0u32)
	}

	fn const_buff<T: bytemuck::Pod>(&self, painter: &mut Painter, data: T) -> Uniform {
		self.create_buff(painter, data).uniform
	}
	fn const_mat3(&self, painter: &mut Painter, mat: Mat3) -> Uniform {
		let u = self.create_mat3(painter);
		u.update_mat3(painter, mat);
		u.uniform
	}
	fn const_mat4(&self, painter: &mut Painter, mat: Mat4) -> Uniform {
		self.create_buff(painter, mat).uniform
	}
	fn const_vec2(&self, painter: &mut Painter, vec: Vec2) -> Uniform {
		self.create_vec2(painter).update(painter, vec);
		self.create_vec2(painter).uniform
	}
	fn const_vec3(&self, painter: &mut Painter, vec: Vec3) -> Uniform {
		let u = self.create_vec3(painter);
		u.update_vec3(painter, vec);
		u.uniform
	}
	fn const_vec4(&self, painter: &mut Painter, vec: Vec4) -> Uniform {
		self.create_buff(painter, vec).uniform
	}
	fn const_uvec2(&self, painter: &mut Painter, vec: UVec2) -> Uniform {
		self.create_buff(painter, vec).uniform
	}
	fn const_f32(&self, painter: &mut Painter, f: f32) -> Uniform {
		self.create_buff(painter, f).uniform
	}
	fn const_u32(&self, painter: &mut Painter, u: u32) -> Uniform {
		self.create_buff(painter, u).uniform
	}
	fn const_tex2d(
		&self,
		painter: &mut Painter,
		texture: Texture,
		sampler: &wgpu::Sampler,
	) -> Uniform {
		self.create_tex2d(painter, texture, sampler).uniform
	}
}

impl UniformType for wgpu::BindGroupLayout {
	fn create_buff<T: bytemuck::Pod>(&self, painter: &mut Painter, data: T) -> UniformBuffer<T> {
		UniformBuffer::new(painter, self, data)
	}

	fn create_tex2d(
		&self,
		painter: &mut Painter,
		texture: Texture,
		sampler: &wgpu::Sampler,
	) -> UniformTex2D {
		UniformTex2D::new(painter, self, texture, sampler)
	}

	fn layout(&self) -> &wgpu::BindGroupLayout {
		self
	}
}
