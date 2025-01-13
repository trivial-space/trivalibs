use crate::{
	painter::get_padded_size,
	texture::{Sampler, Texture},
	Painter,
};
use trivalibs_core::glam::{Mat3, Mat3A, Mat4, Quat, UVec2, Vec2, Vec3, Vec3A, Vec4};
use wgpu::{BindingType, ShaderStages};

#[derive(Clone, Copy)]
pub struct UniformLayout {
	pub(crate) binding_type: BindingType,
	pub(crate) visibility: ShaderStages,
}

impl UniformLayout {
	pub(crate) fn uniform_buffer(visibility: wgpu::ShaderStages) -> Self {
		// TODO: move binding group creation to shade
		// painter
		// 	.device
		// 	.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		// 		entries: &[wgpu::BindGroupLayoutEntry {
		// 			binding: 0,
		// 			visibility,
		// 			ty: wgpu::BindingType::Buffer {
		// 				ty: wgpu::BufferBindingType::Uniform,
		// 				has_dynamic_offset: false,
		// 				min_binding_size: None,
		// 			},
		// 			count: None,
		// 		}],
		// 		label: None,
		// 	});

		Self {
			visibility,
			binding_type: BindingType::Buffer {
				ty: wgpu::BufferBindingType::Uniform,
				has_dynamic_offset: false,
				min_binding_size: None,
			},
		}
	}

	pub(crate) fn tex_2d(visibility: wgpu::ShaderStages) -> Self {
		Self {
			visibility,
			binding_type: wgpu::BindingType::Texture {
				multisampled: false,
				view_dimension: wgpu::TextureViewDimension::D2,
				sample_type: wgpu::TextureSampleType::Float { filterable: true },
			},
		}
	}

	pub(crate) fn sampler(visibility: wgpu::ShaderStages) -> Self {
		Self {
			visibility,
			binding_type: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
		}
	}
}

pub enum Uniform {
	Buffer(Buffer),
	Tex2D(Texture),
	Sampler(Sampler),
}

pub struct UniformBufferType {}

impl UniformBufferType {
	pub fn layout(&self, visibility: wgpu::ShaderStages) -> UniformLayout {
		UniformLayout::uniform_buffer(visibility)
	}

	pub fn vert(&self) -> UniformLayout {
		self.layout(wgpu::ShaderStages::VERTEX)
	}

	pub fn frag(&self) -> UniformLayout {
		self.layout(wgpu::ShaderStages::FRAGMENT)
	}

	pub fn both(&self) -> UniformLayout {
		self.layout(wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT)
	}

	pub fn create_buff<T: bytemuck::Pod>(
		&self,
		painter: &mut Painter,
		data: T,
	) -> UniformBuffer<T> {
		UniformBuffer::new(painter, data)
	}

	pub fn create_mat3(&self, painter: &mut Painter) -> UniformBuffer<Mat3U> {
		self.create_buff(painter, Mat3U(Mat3A::IDENTITY))
	}
	pub fn create_mat4(&self, painter: &mut Painter) -> UniformBuffer<Mat4> {
		self.create_buff(painter, Mat4::IDENTITY)
	}
	pub fn create_vec2(&self, painter: &mut Painter) -> UniformBuffer<Vec2> {
		self.create_buff(painter, Vec2::ZERO)
	}
	pub fn create_vec3(&self, painter: &mut Painter) -> UniformBuffer<Vec3U> {
		self.create_buff(painter, Vec3U(Vec3A::ZERO))
	}
	pub fn create_vec4(&self, painter: &mut Painter) -> UniformBuffer<Vec4> {
		self.create_buff(painter, Vec4::ZERO)
	}
	pub fn create_uvec2(&self, painter: &mut Painter) -> UniformBuffer<UVec2> {
		self.create_buff(painter, UVec2::ZERO)
	}
	pub fn create_f32(&self, painter: &mut Painter) -> UniformBuffer<f32> {
		self.create_buff(painter, 0.0f32)
	}
	pub fn create_u32(&self, painter: &mut Painter) -> UniformBuffer<u32> {
		self.create_buff(painter, 0u32)
	}
	pub fn create_quat(&self, painter: &mut Painter) -> UniformBuffer<Quat> {
		self.create_buff(painter, Quat::IDENTITY)
	}

	pub fn const_buff<T: bytemuck::Pod>(&self, painter: &mut Painter, data: T) -> Uniform {
		self.create_buff(painter, data).uniform()
	}
	pub fn const_mat3(&self, painter: &mut Painter, mat: Mat3) -> Uniform {
		let u = self.create_mat3(painter);
		u.update_mat3(painter, mat);
		u.uniform()
	}
	pub fn const_mat4(&self, painter: &mut Painter, mat: Mat4) -> Uniform {
		self.const_buff(painter, mat)
	}
	pub fn const_vec2(&self, painter: &mut Painter, vec: Vec2) -> Uniform {
		self.const_buff(painter, vec)
	}
	pub fn const_vec3(&self, painter: &mut Painter, vec: Vec3) -> Uniform {
		let u = self.create_vec3(painter);
		u.update_vec3(painter, vec);
		u.uniform()
	}
	pub fn const_vec4(&self, painter: &mut Painter, vec: Vec4) -> Uniform {
		self.const_buff(painter, vec)
	}
	pub fn const_uvec2(&self, painter: &mut Painter, vec: UVec2) -> Uniform {
		self.const_buff(painter, vec)
	}
	pub fn const_f32(&self, painter: &mut Painter, f: f32) -> Uniform {
		self.const_buff(painter, f)
	}
	pub fn const_u32(&self, painter: &mut Painter, u: u32) -> Uniform {
		self.const_buff(painter, u)
	}
	pub fn const_quat(&self, painter: &mut Painter, quat: Quat) -> Uniform {
		self.const_buff(painter, quat)
	}
}

#[derive(Clone, Copy)]
pub struct Buffer(pub(crate) usize);

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
pub struct Mat3U(Mat3A);
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
pub struct Vec3U(Vec3A);
unsafe impl bytemuck::Pod for Vec3U {}

impl UniformBuffer<Vec3U> {
	pub fn new_vec3(painter: &mut Painter, data: Vec3) -> Self {
		UniformBuffer::new(painter, Vec3U(Vec3A::from(data)))
	}

	pub fn update_vec3(&self, painter: &Painter, data: Vec3) {
		self.update(painter, Vec3U(Vec3A::from(data)));
	}
}

pub struct UniformTex2DType {}

impl UniformTex2DType {
	pub fn layout(&self, visibility: wgpu::ShaderStages) -> UniformLayout {
		UniformLayout::tex_2d(visibility)
	}

	pub fn vert(&self) -> UniformLayout {
		self.layout(wgpu::ShaderStages::VERTEX)
	}

	pub fn frag(&self) -> UniformLayout {
		self.layout(wgpu::ShaderStages::FRAGMENT)
	}

	pub fn both(&self) -> UniformLayout {
		self.layout(wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT)
	}

	pub fn uniform(&self, tex: Texture) -> Uniform {
		Uniform::Tex2D(tex)
	}

	// TODO!
	pub fn recreate(&self, _painter: &mut Painter) {
		// let t = &painter.textures[self.texture.0];
		// let s = &painter.samplers[self.sampler.0];
		// let layout = &painter.uniform_types[self.uniform_type.0].layout;

		// let binding = painter
		// 	.device
		// 	.create_bind_group(&wgpu::BindGroupDescriptor {
		// 		layout,
		// 		entries: &[
		// 			wgpu::BindGroupEntry {
		// 				binding: 0,
		// 				resource: wgpu::BindingResource::TextureView(&t.view),
		// 			},
		// 			wgpu::BindGroupEntry {
		// 				binding: 1,
		// 				resource: wgpu::BindingResource::Sampler(&s),
		// 			},
		// 		],
		// 		label: None,
		// 	});

		// if let Uniform::Binding(UniformBinding(idx)) = self.uniform {
		// 	painter.bindings[idx] = binding;
		// }
	}
}

pub struct UniformSamplerType {}

impl UniformSamplerType {
	pub fn layout(&self, visibility: wgpu::ShaderStages) -> UniformLayout {
		UniformLayout::sampler(visibility)
	}

	pub fn vert(&self) -> UniformLayout {
		self.layout(wgpu::ShaderStages::VERTEX)
	}

	pub fn frag(&self) -> UniformLayout {
		self.layout(wgpu::ShaderStages::FRAGMENT)
	}

	pub fn both(&self) -> UniformLayout {
		self.layout(wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT)
	}

	pub fn uniform(&self, sampler: Sampler) -> Uniform {
		Uniform::Sampler(sampler)
	}
}
