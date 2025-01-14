use crate::{
	binding::{Binding, BindingLayout, BindingStorage},
	effect::{Effect, EffectProps, EffectStorage},
	form::{Form, FormBuffers, FormProps, FormStorage},
	layer::{Layer, LayerProps, LayerStorage},
	shade::{AttribsFormat, Shade, ShadeEffectProps, ShadeProps, ShadeStorage},
	shaders::FULL_SCREEN_QUAD,
	shape::{Shape, ShapeProps, ShapeStorage},
	texture::{Sampler, SamplerProps, Texture, Texture2DProps, TextureStorage},
	uniform::{LayerType, UniformBufferType, UniformSamplerType, UniformTex2DType},
};
use std::{collections::BTreeMap, sync::Arc};
use trivalibs_core::utils::default;
use wgpu::{util::make_spirv, ColorTargetState, RenderPassColorAttachment};
use winit::window::Window;

pub(crate) const FULL_SCREEN_TEXTURE_PIPELINE: &'static [u8] = &[0xff, 0xff];

struct PipelineStorage {
	pipeline: wgpu::RenderPipeline,
	uniforms: Vec<Binding>,
}

pub struct Painter {
	pub surface: wgpu::Surface<'static>,
	pub config: wgpu::SurfaceConfiguration,
	pub adapter: wgpu::Adapter,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,

	window: Arc<Window>,
	pub(crate) forms: Vec<FormStorage>,
	pub(crate) shades: Vec<ShadeStorage>,
	pub(crate) textures: Vec<TextureStorage>,
	pub(crate) buffers: Vec<wgpu::Buffer>,
	pub(crate) samplers: Vec<wgpu::Sampler>,
	pub(crate) shapes: Vec<ShapeStorage>,
	pub(crate) effects: Vec<EffectStorage>,
	pub(crate) layers: Vec<LayerStorage>,
	pub(crate) bindings: Vec<BindingStorage>,
	pub(crate) binding_layouts: Vec<wgpu::BindGroupLayout>,
	pub(crate) pipelines: BTreeMap<Vec<u8>, PipelineStorage>,
	fullscreen_quad_shader: wgpu::ShaderModule,
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

		let fullscreen_quad_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: None,
			source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(FULL_SCREEN_QUAD)),
		});

		let mut painter = Self {
			surface,
			config,
			adapter,
			device,
			queue,
			window: window.clone(),
			forms: Vec::with_capacity(8),
			shades: Vec::with_capacity(8),
			textures: Vec::with_capacity(8),
			buffers: Vec::with_capacity(32),
			samplers: Vec::with_capacity(8),
			shapes: Vec::with_capacity(8),
			effects: Vec::with_capacity(8),
			layers: Vec::with_capacity(8),
			binding_layouts: Vec::with_capacity(8),
			bindings: Vec::with_capacity(8),
			pipelines: BTreeMap::new(),
			fullscreen_quad_shader,
		};

		Sampler::create(&mut painter, default());

		let layer_type = LayerType {};
		let layer_layout = BindingLayout::layer(&mut painter, layer_type.frag());

		let fullscreen_quad_pipeline_layout =
			painter
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: None,
					bind_group_layouts: &[&painter.binding_layouts[layer_layout.0]],
					push_constant_ranges: &[],
				});

		let fullscreen_quad_pipeline =
			painter
				.device
				.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
					label: None,
					layout: Some(&fullscreen_quad_pipeline_layout),
					vertex: wgpu::VertexState {
						module: &painter.fullscreen_quad_shader,
						entry_point: Some("vs_main"),
						buffers: &[],
						compilation_options: default(),
					},
					fragment: Some(wgpu::FragmentState {
						module: &painter.fullscreen_quad_shader,
						entry_point: Some("fs_main"),
						targets: &[Some(wgpu::ColorTargetState {
							format: painter.config.format,
							blend: Some(wgpu::BlendState::REPLACE),
							write_mask: wgpu::ColorWrites::ALL,
						})],
						compilation_options: default(),
					}),
					primitive: wgpu::PrimitiveState {
						topology: wgpu::PrimitiveTopology::TriangleStrip,
						strip_index_format: None,
						front_face: wgpu::FrontFace::Cw,
						cull_mode: None,
						polygon_mode: wgpu::PolygonMode::Fill,
						..default()
					},
					depth_stencil: None,
					multisample: wgpu::MultisampleState {
						count: 1,
						mask: !0,
						alpha_to_coverage_enabled: false,
					},
					multiview: None,
					cache: None,
				});

		painter.pipelines.insert(
			FULL_SCREEN_TEXTURE_PIPELINE.to_vec(),
			PipelineStorage {
				pipeline: fullscreen_quad_pipeline,
				uniforms: Vec::with_capacity(0),
			},
		);

		painter
	}

	// form helpers

	pub fn form_update<'a>(&mut self, form: &Form, buffers: impl Into<FormBuffers<'a>>) {
		form.update(self, buffers);
	}

	pub fn form_create<'a>(
		&mut self,
		buffer: impl Into<FormBuffers<'a>>,
		props: FormProps,
	) -> Form {
		Form::new(self, buffer, props)
	}

	// shade helpers

	pub fn shade_create<Format: Into<AttribsFormat>>(
		&mut self,
		props: ShadeProps<Format>,
	) -> Shade {
		Shade::new(self, props)
	}

	pub fn shade_create_effect(&mut self, props: ShadeEffectProps) -> Shade {
		Shade::new_effect(self, props)
	}

	// texture helpers

	pub fn texture_2d_create(&mut self, props: Texture2DProps) -> Texture {
		Texture::create_2d(self, props, false)
	}

	pub fn sampler_create(&mut self, props: SamplerProps) -> Sampler {
		Sampler::create(self, props)
	}

	pub fn sampler_default(&self) -> Sampler {
		Sampler(0)
	}

	// shape utils

	pub fn shape_create(&mut self, form: Form, shade: Shade, props: ShapeProps) -> Shape {
		Shape::new(self, form, shade, props)
	}

	pub fn effect_create(&mut self, shade: Shade, props: EffectProps) -> Effect {
		Effect::new(self, shade, props)
	}

	// layer utils

	pub fn layer_create(&mut self, props: LayerProps) -> Layer {
		Layer::new(self, props)
	}

	// uniform utils

	pub fn uniform_type_buffered(&self) -> UniformBufferType {
		UniformBufferType {}
	}

	pub fn uniform_type_tex_2d(&self) -> UniformTex2DType {
		UniformTex2DType {}
	}

	pub fn uniform_type_sampler(&self) -> UniformSamplerType {
		UniformSamplerType {}
	}

	pub fn uniform_type_layer(&self) -> LayerType {
		LayerType {}
	}

	// general utils

	pub fn request_next_frame(&self) {
		self.window.request_redraw();
	}

	pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		self.config.width = new_size.width.max(1);
		self.config.height = new_size.height.max(1);
		self.surface.configure(&self.device, &self.config);

		let layer_idxs: Vec<usize> = self
			.layers
			.iter()
			.enumerate()
			.filter_map(|(i, l)| if l.use_window_size { Some(i) } else { None })
			.collect();

		for idx in layer_idxs {
			Layer(idx).resize(self, 0, 0);
		}
	}

	pub fn canvas_size(&self) -> winit::dpi::PhysicalSize<u32> {
		self.window.inner_size()
	}

	fn set_shape_pipeline(
		&mut self,
		rpass: &mut wgpu::RenderPass,
		shape: Shape,
		layer: Option<Layer>,
	) {
		let l = layer.map(|l| &self.layers[l.0]);

		let layer_key = match l {
			Some(layer) => layer.pipeline_key.as_slice(),
			None => &[],
		};

		let sp = &self.shapes[shape.0];
		let sd = &self.shades[sp.shade.0];
		let pipeline_key = &[sp.pipeline_key.as_slice(), layer_key].concat();

		if !self.pipelines.contains_key(pipeline_key) {
			let f = &self.forms[sp.form.0];

			let targets: Vec<Option<ColorTargetState>> = if let Some(l) = l {
				l.formats
					.iter()
					.map(|f| {
						Some(wgpu::ColorTargetState {
							format: *f,
							blend: Some(sp.blend_state),
							write_mask: wgpu::ColorWrites::ALL,
						})
					})
					.collect::<Vec<_>>()
			} else {
				vec![Some(wgpu::ColorTargetState {
					format: self.config.format,
					blend: Some(sp.blend_state),
					write_mask: wgpu::ColorWrites::ALL,
				})]
			};

			let vertex_shader = self
				.device
				.create_shader_module(wgpu::ShaderModuleDescriptor {
					label: None,
					source: make_spirv(&sd.vertex_bytes.as_ref().unwrap()),
				});

			let fragment_shader = self
				.device
				.create_shader_module(wgpu::ShaderModuleDescriptor {
					label: None,
					source: make_spirv(&sd.fragment_bytes.as_ref().unwrap()),
				});

			let pipeline = self
				.device
				.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
					label: None,
					layout: Some(&sd.pipeline_layout),
					vertex: wgpu::VertexState {
						module: &vertex_shader,
						entry_point: None,
						buffers: &[wgpu::VertexBufferLayout {
							array_stride: sd.attribs.stride,
							step_mode: wgpu::VertexStepMode::Vertex,
							attributes: &sd.attribs.attributes,
						}],
						compilation_options: default(),
					},
					fragment: Some(wgpu::FragmentState {
						module: &fragment_shader,
						entry_point: None,
						targets: targets.as_slice(),
						compilation_options: default(),
					}),
					primitive: wgpu::PrimitiveState {
						topology: f.props.topology,
						strip_index_format: None,
						front_face: f.props.front_face,
						cull_mode: sp.cull_mode,
						// Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
						polygon_mode: wgpu::PolygonMode::Fill,
						unclipped_depth: false,
						conservative: false,
					},
					depth_stencil: if l.map_or(false, |l| l.depth_texture.is_some()) {
						Some(wgpu::DepthStencilState {
							format: wgpu::TextureFormat::Depth24Plus,
							depth_write_enabled: true,
							depth_compare: wgpu::CompareFunction::Less,
							stencil: default(),
							bias: default(),
						})
					} else {
						None
					},
					multisample: wgpu::MultisampleState {
						count: l.map_or(1, |l| {
							if l.multisampled_textures.is_empty() {
								1
							} else {
								4
							}
						}),
						mask: !0,
						alpha_to_coverage_enabled: false,
					},
					multiview: None,
					cache: None,
				});

			let l = layer.map(|l| &self.layers[l.0]);
			let sp = &self.shapes[shape.0];
			let data = &sp.data.clone();
			let instances = &sp.instances.clone();
			let layer_data = l.and_then(|l| l.data.clone());

			let pipeline = PipelineStorage {
				pipeline,
				uniforms: Binding::uniforms(
					self,
					sd.uniforms_length,
					sd.uniform_layout,
					data,
					instances,
					&layer_data,
				),
			};

			self.pipelines.insert(pipeline_key.clone(), pipeline);
		}

		let pipeline = &self.pipelines[pipeline_key];
		rpass.set_pipeline(&pipeline.pipeline);
	}

	fn set_effect_pipeline(&mut self, rpass: &mut wgpu::RenderPass, effect: Effect, layer: Layer) {
		let layer = &self.layers[layer.0];
		let effect = &self.effects[effect.0];

		let layer_key = layer.pipeline_key.as_slice();
		let pipeline_key = &[effect.pipeline_key.as_slice(), layer_key].concat();

		if !self.pipelines.contains_key(pipeline_key) {
			let s = &self.shades[effect.shade.0];

			let fragment_shader = self
				.device
				.create_shader_module(wgpu::ShaderModuleDescriptor {
					label: None,
					source: make_spirv(&s.fragment_bytes.as_ref().unwrap()),
				});

			let targets: Vec<Option<ColorTargetState>> = layer
				.formats
				.iter()
				.map(|f| {
					Some(wgpu::ColorTargetState {
						format: *f,
						blend: Some(effect.blend_state),
						write_mask: wgpu::ColorWrites::ALL,
					})
				})
				.collect::<Vec<_>>();

			let pipeline = self
				.device
				.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
					label: None,
					layout: Some(&s.pipeline_layout),
					vertex: wgpu::VertexState {
						module: &self.fullscreen_quad_shader,
						entry_point: Some("vs_main"),
						buffers: &[],
						compilation_options: default(),
					},
					fragment: Some(wgpu::FragmentState {
						module: &fragment_shader,
						entry_point: None,
						targets: targets.as_slice(),
						compilation_options: default(),
					}),
					primitive: wgpu::PrimitiveState {
						topology: wgpu::PrimitiveTopology::TriangleStrip,
						strip_index_format: None,
						front_face: wgpu::FrontFace::Cw,
						cull_mode: None,
						polygon_mode: wgpu::PolygonMode::Fill,
						..default()
					},
					depth_stencil: None,
					multisample: wgpu::MultisampleState {
						count: 1,
						mask: !0,
						alpha_to_coverage_enabled: false,
					},
					multiview: None,
					cache: None,
				});

			self.pipelines.insert(pipeline_key.to_vec(), pipeline);
		}

		let pipeline = &self.pipelines[pipeline_key];
		rpass.set_pipeline(pipeline);
	}

	fn render_shape(
		&mut self,
		rpass: &mut wgpu::RenderPass<'_>,
		shape: Shape,
		layer: Option<Layer>,
	) {
		self.set_shape_pipeline(rpass, shape, layer);

		let shape = &self.shapes[shape.0];
		let form = &self.forms[shape.form.0];

		let binding_index = shape.uniform_binding_index;

		let draw = |rpass: &mut wgpu::RenderPass, binding: Option<Binding>| {
			if let Some(layer) = layer {
				let l = &self.layers[layer.0];
				if let Some(data) = l.data {
					for (index, layer) in data.layers {
						let l = &self.layers[layer.0];
						let b = l.current_source();
						rpass.set_bind_group(index, &self.bindings[b.0].binding, &[]);
					}
				}
			}

			if let Some(data) = shape.data {
				for (index, layer) in data.layers {
					let l = &self.layers[layer.0];
					let b = l.current_source();
					rpass.set_bind_group(index, &self.bindings[b.0].binding, &[]);
				}
			}

			if let Some(binding) = binding {
				rpass.set_bind_group(binding_index, &self.bindings[binding.0].binding, &[]);
			}

			rpass.set_vertex_buffer(0, form.vertex_buffer.slice(..));
			if let Some(index_buffer) = &form.index_buffer {
				rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
				rpass.draw_indexed(0..form.index_count, 0, 0..1);
			} else {
				rpass.draw(0..form.vertex_count, 0..1);
			}
		};

		if shape.instances.len() > 0 {
			for uniforms in &shape.instances {
				for (index, uniform) in uniforms {
					rpass.set_bind_group(*index, uniform.binding(self), &[]);
				}
				draw(rpass);
			}
		} else {
			draw(rpass);
		}
	}

	fn render_effect(
		&mut self,
		effect: Effect,
		layer: Layer,
		skip_source: bool,
	) -> Result<(), wgpu::SurfaceError> {
		let l = &self.layers[layer.0];

		let view = &self.textures[l.current_target().0].view;

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

		{
			let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: None,
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: l
							.clear_color
							.map_or(wgpu::LoadOp::Load, |color| wgpu::LoadOp::Clear(color)),
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
			});

			self.set_effect_pipeline(&mut rpass, effect, layer);

			let e = &self.effects[effect.0];

			if !skip_source {
				let tex = self.layers[layer.0].current_source();
				rpass.set_bind_group(0, tex.uniform.binding(self), &[]);
			}

			let l = &self.layers[layer.0];
			for (index, uniform) in &l.uniforms {
				rpass.set_bind_group(*index, uniform.binding(self), &[]);
			}

			for (index, uniform) in &e.uniforms {
				rpass.set_bind_group(*index, uniform.binding(self), &[]);
			}

			if !e.instances.is_empty() {
				for uniforms in &e.instances {
					for (index, uniform) in uniforms {
						rpass.set_bind_group(*index, uniform.binding(self), &[]);
					}
					rpass.draw(0..3, 0..1);
				}
			} else {
				rpass.draw(0..3, 0..1);
			}
		}

		self.queue.submit(Some(encoder.finish()));

		Ok(())
	}

	pub fn draw<'a>(&mut self, shape: Shape) -> Result<(), wgpu::SurfaceError> {
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

			self.render_shape(&mut rpass, shape, None);
		}

		self.queue.submit(Some(encoder.finish()));
		frame.present();

		Ok(())
	}

	pub fn paint(&mut self, layer: Layer) -> Result<(), wgpu::SurfaceError> {
		let l = &self.layers[layer.0];
		let shapes_len = l.shapes.len();
		let effects_len = l.effects.len();
		let has_shapes = shapes_len > 0;

		if has_shapes {
			let color_attachments: Vec<Option<RenderPassColorAttachment<'_>>> = if !l
				.is_multi_target
			{
				let target_view = &self.textures[l.current_target().0].view;
				let multisampled_texture = l.multisampled_textures.get(0);

				let view = multisampled_texture.map_or(target_view, |t| &self.textures[t.0].view);
				let resolve_target = multisampled_texture.map(|_| target_view);

				vec![Some(wgpu::RenderPassColorAttachment {
					view,
					resolve_target,
					ops: wgpu::Operations {
						load: l
							.clear_color
							.map_or(wgpu::LoadOp::Load, |color| wgpu::LoadOp::Clear(color)),
						store: wgpu::StoreOp::Store,
					},
				})]
			} else {
				l.target_textures
					.iter()
					.enumerate()
					.map(|(i, t)| {
						let target_view = &self.textures[t.0].view;
						let multisampled_texture = l.multisampled_textures.get(i);

						let view =
							multisampled_texture.map_or(target_view, |t| &self.textures[t.0].view);
						let resolve_target = multisampled_texture.map(|_| target_view);

						Some(wgpu::RenderPassColorAttachment {
							view,
							resolve_target,
							ops: wgpu::Operations {
								load: l
									.clear_color
									.map_or(wgpu::LoadOp::Load, |color| wgpu::LoadOp::Clear(color)),
								store: wgpu::StoreOp::Store,
							},
						})
					})
					.collect::<Vec<_>>()
			};

			let mut encoder = self
				.device
				.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

			{
				let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
					label: None,
					color_attachments: &color_attachments,
					depth_stencil_attachment: l.depth_texture.as_ref().map(|t| {
						wgpu::RenderPassDepthStencilAttachment {
							view: &self.textures[t.0].view,
							depth_ops: Some(wgpu::Operations {
								load: wgpu::LoadOp::Clear(1.0),
								store: wgpu::StoreOp::Store,
							}),
							stencil_ops: None,
						}
					}),
					timestamp_writes: None,
					occlusion_query_set: None,
				});

				for i in 0..shapes_len {
					let shape = self.layers[layer.0].shapes[i];
					self.render_shape(&mut rpass, shape, Some(layer));
				}
			}

			self.queue.submit(Some(encoder.finish()));
		}

		if effects_len == 0 {
			return Ok(());
		}

		if has_shapes {
			self.layers[layer.0].swap_targets();
		}

		for i in 0..effects_len {
			let skip_source_tex = i == 0 && !has_shapes;
			let effect = self.layers[layer.0].effects[i];
			self.render_effect(effect, layer, skip_source_tex)?;
			self.layers[layer.0].swap_targets();
		}

		Ok(())
	}

	pub fn compose(&mut self, layers: &[Layer]) -> Result<(), wgpu::SurfaceError> {
		for layer in layers {
			self.paint(*layer)?;
		}
		Ok(())
	}

	pub fn show(&mut self, layer: Layer) -> Result<(), wgpu::SurfaceError> {
		let frame = self.surface.get_current_texture()?;

		let view = frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

		let layer_binding = self.layers[layer.0].current_source();
		let pipeline = &self.pipelines[FULL_SCREEN_TEXTURE_PIPELINE];

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
			rpass.set_pipeline(pipeline);
			rpass.set_bind_group(0, &self.bindings[layer_binding.0].binding, &[]);
			rpass.draw(0..3, 0..1);
		}

		self.queue.submit(Some(encoder.finish()));
		frame.present();

		Ok(())
	}

	pub(crate) fn reload_shader(&mut self, path: String) {
		println!("Reloading shader: {}", path);
		let shade_indices = self
			.shades
			.iter()
			.enumerate()
			.filter_map(|(idx, s)| {
				if s.vertex_path.as_ref().map_or(false, |p| p.contains(&path)) {
					return Some(idx);
				}
				if s.fragment_path
					.as_ref()
					.map_or(false, |p| p.contains(&path))
				{
					return Some(idx);
				}
				None
			})
			.collect::<Vec<_>>();

		let pipeline_keys = self
			.pipelines
			.keys()
			.cloned()
			.map(|key| (u16::from_le_bytes([key[0], key[1]]), key))
			.collect::<Vec<_>>();

		for idx in shade_indices {
			Shade(idx).load_fragment_from_path(self);
			Shade(idx).load_vertex_from_path(self);

			for (shade_idx, pipeline_key) in &pipeline_keys {
				if *shade_idx == idx as u16 {
					self.pipelines.remove(pipeline_key);
				}
			}
		}
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
