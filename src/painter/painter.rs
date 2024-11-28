use std::{collections::HashMap, sync::Arc};

use wgpu::{Adapter, Device, Queue, Surface, SurfaceConfiguration};
use winit::window::Window;

use crate::{rendering::RenderableBuffer, utils::default};

struct FormStorage {
	vertex_buffer: wgpu::Buffer,
	index_buffer: Option<wgpu::Buffer>,
	vertex_count: u32,
	index_count: u32,
}

#[derive(Clone, Copy)]
pub struct Form(usize);

pub struct FormProps<'a, T>
where
	T: bytemuck::Pod + bytemuck::Zeroable,
{
	pub vertex_buffer: &'a [T],
	pub index_buffer: Option<&'a [u32]>,
}

pub struct ShadeStorage {
	pipeline: wgpu::RenderPipeline,
}

#[derive(Clone, Copy)]
pub struct Shade(usize);

pub struct ShadeProps<'a, Format: Into<FormFormat>> {
	pub vertex_shader: wgpu::ShaderModuleDescriptor<'a>,
	pub fragment_shader: wgpu::ShaderModuleDescriptor<'a>,
	pub vertex_format: Format,
	pub uniform_layout: &'a [&'a wgpu::BindGroupLayout],
}

pub struct FormFormat {
	stride: u64,
	attributes: Vec<wgpu::VertexAttribute>,
}

pub fn attrib(location: u32, format: wgpu::VertexFormat, offset: u64) -> wgpu::VertexAttribute {
	wgpu::VertexAttribute {
		shader_location: location,
		format,
		offset,
	}
}

impl Into<FormFormat> for &[wgpu::VertexFormat] {
	fn into(self) -> FormFormat {
		let mut stride = 0;
		let mut attributes = Vec::with_capacity(self.len());
		let mut location = 0;
		for format in self {
			attributes.push(attrib(location, *format, stride));
			stride += format.size();
			location += 1;
		}

		FormFormat { attributes, stride }
	}
}

impl Into<FormFormat> for Vec<wgpu::VertexFormat> {
	fn into(self) -> FormFormat {
		self.as_slice().into()
	}
}

impl Into<FormFormat> for wgpu::VertexFormat {
	fn into(self) -> FormFormat {
		FormFormat {
			attributes: vec![attrib(0, self, 0)],
			stride: self.size(),
		}
	}
}

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
	forms: Vec<FormStorage>,
	shades: Vec<ShadeStorage>,
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
		}
	}

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

	pub fn update_form<T>(&mut self, form: &Form, props: &FormProps<T>)
	where
		T: bytemuck::Pod,
	{
		let f = &mut self.forms[form.0];

		f.vertex_count = props.vertex_buffer.len() as u32;

		self.queue.write_buffer(
			&f.vertex_buffer,
			0,
			bytemuck::cast_slice(props.vertex_buffer),
		);

		if let Some(index_data) = props.index_buffer {
			f.index_count = index_data.len() as u32;

			let index_buffer =
				f.index_buffer
					.get_or_insert(self.device.create_buffer(&wgpu::BufferDescriptor {
						label: None,
						usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
						size: get_padded_size(
							index_data.len() as u64 * std::mem::size_of::<u32>() as u64,
						),
						mapped_at_creation: false,
					}));

			self.queue.write_buffer(
				index_buffer,
				0,
				bytemuck::cast_slice(props.index_buffer.unwrap()),
			);
		}
	}

	pub fn update_form_buffer(&mut self, form: &Form, buffers: RenderableBuffer) {
		let f = &mut self.forms[form.0];

		f.vertex_count = buffers.vertex_count;

		self.queue
			.write_buffer(&f.vertex_buffer, 0, &buffers.vertex_buffer);

		if let Some(index_data) = buffers.index_buffer {
			f.index_count = buffers.index_count;

			let index_buffer =
				f.index_buffer
					.get_or_insert(self.device.create_buffer(&wgpu::BufferDescriptor {
						label: None,
						usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
						size: get_padded_size(
							buffers.index_count as u64 * std::mem::size_of::<u32>() as u64,
						),
						mapped_at_creation: false,
					}));

			self.queue.write_buffer(index_buffer, 0, &index_data);
		}
	}

	pub fn create_form_with_size(&mut self, size: u64) -> Form {
		let vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
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

		let i = self.forms.len();
		self.forms.push(f);

		return Form(i);
	}

	pub fn create_form<T>(&mut self, props: &FormProps<T>) -> Form
	where
		T: bytemuck::Pod,
	{
		let form = self.create_form_with_size(
			props.vertex_buffer.len() as u64 * std::mem::size_of::<T>() as u64,
		);

		self.update_form(&form, props);

		form
	}

	pub fn create_shade<Format: Into<FormFormat>>(&mut self, props: ShadeProps<Format>) -> Shade {
		let pipeline_layout = self
			.device
			.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: None,
				bind_group_layouts: props.uniform_layout,
				push_constant_ranges: &[],
			});

		let format = props.vertex_format.into();

		let vertex_shader = self.device.create_shader_module(props.vertex_shader);
		let fragment_shader = self.device.create_shader_module(props.fragment_shader);

		let pipeline = self
			.device
			.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				label: None,
				layout: Some(&pipeline_layout),
				vertex: wgpu::VertexState {
					module: &vertex_shader,
					entry_point: None,
					buffers: &[wgpu::VertexBufferLayout {
						array_stride: format.stride,
						step_mode: wgpu::VertexStepMode::Vertex,
						attributes: &format.attributes,
					}],
					compilation_options: default(),
				},
				fragment: Some(wgpu::FragmentState {
					module: &fragment_shader,
					entry_point: None,
					targets: &[Some(wgpu::ColorTargetState {
						format: self.config.format,
						blend: Some(wgpu::BlendState::REPLACE),
						write_mask: wgpu::ColorWrites::ALL,
					})],
					compilation_options: default(),
				}),
				primitive: wgpu::PrimitiveState {
					topology: wgpu::PrimitiveTopology::TriangleList,
					strip_index_format: None,
					front_face: wgpu::FrontFace::Ccw,
					cull_mode: Some(wgpu::Face::Back),
					// Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
					polygon_mode: wgpu::PolygonMode::Fill,
					unclipped_depth: false,
					conservative: false,
				},
				depth_stencil: None,
				multisample: wgpu::MultisampleState::default(),
				multiview: None,
				cache: None,
			});

		let s = ShadeStorage { pipeline };

		let i = self.shades.len();
		self.shades.push(s);

		Shade(i)
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
		uniforms: HashMap<u32, impl Into<Option<&'a wgpu::BindGroup>>>,
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
				rpass.set_bind_group(index, bind_group.into(), &[]);
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

fn get_padded_size(unpadded_size: u64) -> u64 {
	// Valid vulkan usage is
	// 1. buffer size must be a multiple of COPY_BUFFER_ALIGNMENT.
	// 2. buffer size must be greater than 0.
	// Therefore we round the value up to the nearest multiple, and ensure it's at least COPY_BUFFER_ALIGNMENT.
	let align_mask = wgpu::COPY_BUFFER_ALIGNMENT - 1;
	((unpadded_size + align_mask) & !align_mask).max(wgpu::COPY_BUFFER_ALIGNMENT)
}
