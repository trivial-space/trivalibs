use super::{
	form::{Form, FormProps, FormStorage},
	shade::{FormFormat, Shade, ShadeProps, ShadeStorage},
	uniform::{get_uniform_layout_buffered, Uniform},
};
use crate::{rendering::RenderableBuffer, utils::default};
use glam::{Mat3, Mat3A, Mat4};
use std::{collections::HashMap, sync::Arc};
use wgpu::{Adapter, Device, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

pub struct Texture2DProps {
	pub width: u32,
	pub height: u32,
	pub format: wgpu::TextureFormat,
	pub usage: wgpu::TextureUsages,
}

pub struct SamplerProps {
	pub address_mode_u: wgpu::AddressMode,
	pub address_mode_v: wgpu::AddressMode,
	pub mag_filter: wgpu::FilterMode,
	pub min_filter: wgpu::FilterMode,
}

pub struct Painter {
	pub surface: Surface<'static>,
	pub config: SurfaceConfiguration,
	pub adapter: Adapter,
	pub device: Device,
	pub queue: Queue,
	window: Arc<Window>,
	pub(crate) forms: Vec<FormStorage>,
	pub(crate) shades: Vec<ShadeStorage>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable)]
pub struct Mat3U(Mat3A);
unsafe impl bytemuck::Pod for Mat3U {}

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
		}
	}

	// form helpers

	pub fn update_form<T>(&mut self, form: &Form, props: &FormProps<T>)
	where
		T: bytemuck::Pod,
	{
		form.update_form(self, props);
	}

	pub fn update_form_buffer(&mut self, form: &Form, buffers: RenderableBuffer) {
		form.update_form_buffer(self, buffers);
	}

	pub fn create_form_with_size(&mut self, size: u64) -> Form {
		Form::new_with_size(self, size)
	}

	pub fn create_form<T>(&mut self, props: &FormProps<T>) -> Form
	where
		T: bytemuck::Pod,
	{
		Form::new(self, props)
	}

	// shade helpers

	pub fn create_shade<Format: Into<FormFormat>>(&mut self, props: ShadeProps<Format>) -> Shade {
		Shade::new(self, props)
	}

	// uniform utile

	pub fn create_uniform_buffered<T>(&self, layout: &wgpu::BindGroupLayout, data: T) -> Uniform<T>
	where
		T: bytemuck::Pod,
	{
		Uniform::new_buffered(self, layout, data)
	}

	pub fn update_uniform_buffered<T>(&self, uniform: &Uniform<T>, data: T)
	where
		T: bytemuck::Pod,
	{
		uniform.update_buffered(self, data);
	}

	pub fn create_uniform_mat4(&self, layout: &wgpu::BindGroupLayout, mat: Mat4) -> Uniform<Mat4> {
		self.create_uniform_buffered(layout, mat)
	}

	pub fn update_uniform_mat4(&self, uniform: &Uniform<Mat4>, mat: Mat4) {
		self.update_uniform_buffered(uniform, mat);
	}

	pub fn create_uniform_mat3(&self, layout: &wgpu::BindGroupLayout, mat: Mat3) -> Uniform<Mat3U> {
		self.create_uniform_buffered(layout, Mat3U(Mat3A::from(mat)))
	}

	pub fn update_uniform_mat3(&self, uniform: &Uniform<Mat3U>, mat: Mat3) {
		self.update_uniform_buffered(uniform, Mat3U(Mat3A::from(mat)));
	}

	pub fn get_uniform_layout_buffered(
		&self,
		visibility: wgpu::ShaderStages,
	) -> wgpu::BindGroupLayout {
		get_uniform_layout_buffered(self, visibility)
	}

	// general utils

	pub fn redraw(&self) {
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

	pub fn fill_texture_2d(&self, texture: &wgpu::Texture, data: &[u8]) {
		let size = texture.size();
		self.queue.write_texture(
			// Tells wgpu where to copy the pixel data
			wgpu::ImageCopyTexture {
				texture: texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			// The actual pixel data
			data,
			// The layout of the texture
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4 * size.width),
				rows_per_image: Some(size.height),
			},
			size,
		);
	}

	pub fn create_texture_2d(&mut self, props: &Texture2DProps) -> wgpu::Texture {
		let texture_size = wgpu::Extent3d {
			width: props.width,
			height: props.height,
			depth_or_array_layers: 1,
		};

		self.device.create_texture(&wgpu::TextureDescriptor {
			size: texture_size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: props.format,
			usage: props.usage,
			label: None,
			// The base format (Rgba8UnormSrgb) is
			// always supported. Note that using a different
			// texture format is not supported on the WebGL2
			// backend.
			view_formats: &[],
		})
	}

	pub fn get_texture_2d_uniform(
		&self,
		texture: &wgpu::Texture,
		sampler: &wgpu::Sampler,
	) -> wgpu::BindGroup {
		let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

		self.device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &self.get_texture_2d_uniform_layout(),
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(sampler),
				},
			],
			label: None,
		})
	}

	pub fn get_texture_2d_uniform_layout(&self) -> wgpu::BindGroupLayout {
		self.device
			.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							multisampled: false,
							view_dimension: wgpu::TextureViewDimension::D2,
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::FRAGMENT,
						// This should match the filterable field of the
						// corresponding Texture entry above.
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
						count: None,
					},
				],
				label: None,
			})
	}

	pub fn create_sampler(&self, props: &SamplerProps) -> wgpu::Sampler {
		self.device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: props.address_mode_u,
			address_mode_v: props.address_mode_v,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: props.mag_filter,
			min_filter: props.min_filter,
			mipmap_filter: wgpu::FilterMode::Nearest,
			compare: None,
			..default()
		})
	}

	pub fn draw<'a>(
		&self,
		form: &Form,
		shade: &Shade,
		uniforms: HashMap<u32, &'a wgpu::BindGroup>,
	) -> std::result::Result<(), wgpu::SurfaceError> {
		let f = &self.forms[form.0];
		let s = &self.shades[shade.0];

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
			rpass.set_pipeline(&s.pipeline);
			for (index, bind_group) in uniforms {
				rpass.set_bind_group(index, bind_group, &[]);
			}
			rpass.set_vertex_buffer(0, f.vertex_buffer.slice(..));
			if let Some(index_buffer) = &f.index_buffer {
				rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
				rpass.draw_indexed(0..f.index_count, 0, 0..1);
			} else {
				rpass.draw(0..f.vertex_count, 0..1);
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
