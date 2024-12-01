use super::{
	form::{Form, FormData, FormProps, FormStorage},
	shade::{AttribsFormat, Shade, ShadeProps, ShadeStorage},
	sketch::{Sketch, SketchProps, SketchStorage},
	texture::{SamplerProps, Texture, Texture2DProps, TextureStorage, UniformTex2D},
	uniform::{get_uniform_layout_buffered, Mat3U, UniformBuffer},
};
use crate::rendering::RenderableBuffer;
use glam::{Mat3, Mat4};
use std::sync::Arc;
use wgpu::{Adapter, BindGroupLayout, Device, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

pub struct Painter {
	pub surface: Surface<'static>,
	pub config: SurfaceConfiguration,
	pub adapter: Adapter,
	pub device: Device,
	pub queue: Queue,
	window: Arc<Window>,
	pub(crate) forms: Vec<FormStorage>,
	pub(crate) shades: Vec<ShadeStorage>,
	pub(crate) textures: Vec<TextureStorage>,
	pub(crate) sketches: Vec<SketchStorage>,
	pub(crate) bindings: Vec<wgpu::BindGroup>,
}

impl Painter {
	pub async fn new(window: Arc<Window>) -> Self {
		let mut size = window.inner_size();
		size.width = size.width.max(1);
		size.height = size.height.max(1);

		let instance = wgpu::Instance::default();

		let surface = instance.create_surface(window.clone()).unwrap();
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				force_fallback_adapter: false,
				// Request an adapter which can render to our surface
				compatible_surface: Some(&surface),
			})
			.await
			.expect("Failed to find an appropriate adapter");

		// Create the logical device and command queue
		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: None,
					required_features: wgpu::Features::empty(),
					// Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
					required_limits: wgpu::Limits::downlevel_webgl2_defaults()
						.using_resolution(adapter.limits()),
					memory_hints: wgpu::MemoryHints::MemoryUsage,
				},
				None,
			)
			.await
			.expect("Failed to create device");

		// We could also manually create a SurfaceConfiguration.
		// See https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#state-new for example.
		let config = surface
			.get_default_config(&adapter, size.width, size.height)
			.unwrap();

		surface.configure(&device, &config);

		Self {
			surface,
			config,
			adapter,
			device,
			queue,
			window: window.clone(),
			forms: Vec::with_capacity(8),
			shades: Vec::with_capacity(8),
			textures: Vec::with_capacity(8),
			sketches: Vec::with_capacity(8),
			bindings: Vec::with_capacity(8),
		}
	}

	// form helpers

	pub fn form_update<T>(&mut self, form: &Form, props: &FormData<T>)
	where
		T: bytemuck::Pod,
	{
		form.update(self, props);
	}

	pub fn form_update_buffer(&mut self, form: &Form, buffers: RenderableBuffer) {
		form.update_buffer(self, buffers);
	}

	pub fn form_create_with_size(&mut self, size: u64, props: FormProps) -> Form {
		Form::new_with_size(self, size, props)
	}

	pub fn form_create<T>(&mut self, data: &FormData<T>, props: FormProps) -> Form
	where
		T: bytemuck::Pod,
	{
		Form::new(self, data, props)
	}

	pub fn from_from_buffer(&mut self, buffer: RenderableBuffer, props: FormProps) -> Form {
		Form::from_buffer(self, buffer, props)
	}

	// shade helpers

	pub fn shade_create<Format: Into<AttribsFormat>>(
		&mut self,
		props: ShadeProps<Format>,
	) -> Shade {
		Shade::new(self, props)
	}

	// uniform utile

	pub fn uniform_create_buffered<T>(
		&mut self,
		layout: &wgpu::BindGroupLayout,
		data: T,
	) -> UniformBuffer<T>
	where
		T: bytemuck::Pod,
	{
		UniformBuffer::new_buffered(self, layout, data)
	}

	pub fn uniform_update_buffered<T>(&self, uniform: &UniformBuffer<T>, data: T)
	where
		T: bytemuck::Pod,
	{
		uniform.update_buffered(self, data);
	}

	pub fn uniform_create_mat4(
		&mut self,
		layout: &wgpu::BindGroupLayout,
		mat: Mat4,
	) -> UniformBuffer<Mat4> {
		self.uniform_create_buffered(layout, mat)
	}

	pub fn uniform_update_mat4(&self, uniform: &UniformBuffer<Mat4>, mat: Mat4) {
		self.uniform_update_buffered(uniform, mat);
	}

	pub fn uniform_create_mat3(
		&mut self,
		layout: &wgpu::BindGroupLayout,
		mat: Mat3,
	) -> UniformBuffer<Mat3U> {
		UniformBuffer::new_mat3(self, layout, mat)
	}

	pub fn uniform_update_mat3(&self, uniform: &UniformBuffer<Mat3U>, mat: Mat3) {
		uniform.update_mat3(self, mat);
	}

	pub fn uniform_get_layout_buffered(
		&self,
		visibility: wgpu::ShaderStages,
	) -> wgpu::BindGroupLayout {
		get_uniform_layout_buffered(self, visibility)
	}

	// texture helpers

	pub fn texture_2d_fill(&self, texture: Texture, data: &[u8]) {
		texture.fill_2d(self, data);
	}

	pub fn texture_2d_create(&mut self, props: &Texture2DProps) -> Texture {
		Texture::create_2d(self, props)
	}

	pub fn texture_2d_get_uniform_layout(
		&self,
		visibility: wgpu::ShaderStages,
	) -> wgpu::BindGroupLayout {
		Texture::get_2d_uniform_layout(self, visibility)
	}

	pub fn uniform_create_tex(
		&mut self,
		layout: &BindGroupLayout,
		texture: Texture,
		sampler: &wgpu::Sampler,
	) -> UniformTex2D {
		texture.get_uniform(self, layout, sampler)
	}

	pub fn create_sampler(&self, props: &SamplerProps) -> wgpu::Sampler {
		Texture::create_sampler(self, props)
	}

	// sketch utils

	pub fn sketch_create(&mut self, form: Form, shade: Shade, props: &SketchProps) -> Sketch {
		Sketch::new(self, form, shade, props)
	}

	// general utils

	pub fn request_redraw(&self) {
		self.window.request_redraw();
	}

	pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		self.config.width = new_size.width.max(1);
		self.config.height = new_size.height.max(1);
		self.surface.configure(&self.device, &self.config);
	}

	pub fn canvas_size(&self) -> winit::dpi::PhysicalSize<u32> {
		self.window.inner_size()
	}

	pub fn draw<'a>(&self, sketch: &Sketch) -> std::result::Result<(), wgpu::SurfaceError> {
		let sketch = &self.sketches[sketch.0];
		let form = &self.forms[sketch.form.0];

		let frame = self.surface.get_current_texture()?;

		let view = frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

		{
			let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: None,
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
			});
			rpass.set_pipeline(&sketch.pipeline);
			for (index, uniform) in &sketch.uniforms {
				let binding = &self.bindings[uniform.0];
				rpass.set_bind_group(*index, binding, &[]);
			}
			rpass.set_vertex_buffer(0, form.vertex_buffer.slice(..));
			if let Some(index_buffer) = &form.index_buffer {
				rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
				rpass.draw_indexed(0..form.index_count, 0, 0..1);
			} else {
				rpass.draw(0..form.vertex_count, 0..1);
			}
		}

		self.queue.submit(Some(encoder.finish()));
		frame.present();

		Ok(())
	}
}

pub(crate) fn get_padded_size(unpadded_size: u64) -> u64 {
	// Valid vulkan usage is
	// 1. buffer size must be a multiple of COPY_BUFFER_ALIGNMENT.
	// 2. buffer size must be greater than 0.
	// Therefore we round the value up to the nearest multiple, and ensure it's at least COPY_BUFFER_ALIGNMENT.
	let align_mask = wgpu::COPY_BUFFER_ALIGNMENT - 1;
	((unpadded_size + align_mask) & !align_mask).max(wgpu::COPY_BUFFER_ALIGNMENT)
}
