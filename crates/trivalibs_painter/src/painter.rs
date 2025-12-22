use crate::{
	bind_group::{BindGroup, BindGroupLayout, BindGroupStorage, LayerBindGroupData},
	binding::{BindingBuffer, LayerBinding, Mat3U, ValueBinding, Vec3U},
	effect::{Effect, EffectBuilder, EffectStorage},
	form::{Form, FormBuffer, FormBuilder, FormStorage},
	layer::{
		InstanceRenderingStrategy, Layer, LayerBuilder, LayerStorage, SingleEffectLayerBuilder,
	},
	pipeline::PipelineStorage,
	prelude::{BINDING_LAYER_FRAG, BINDING_SAMPLER_FRAG},
	sampler::{Sampler, SamplerBuilder, SamplerProps},
	shade::{AttribsFormat, Shade, ShadeBuilder, ShadeEffectBuilder, ShadeStorage},
	shaders::FULL_SCREEN_QUAD,
	shape::{Shape, ShapeBuilder, ShapeStorage},
	texture::{TexViewKey, TextureStorage},
};
use std::{collections::BTreeMap, sync::Arc};
use trivalibs_core::{
	glam::{Mat3, Mat3A, Mat4, Quat, UVec2, Vec2, Vec3, Vec3A, Vec4},
	utils::default,
};
use wgpu::RenderPassColorAttachment;
use winit::window::Window;

pub(crate) const FULL_SCREEN_TEXTURE_PIPELINE: &'static [u8] = &[0xff, 0xff];

pub struct Painter {
	pub surface: wgpu::Surface<'static>,
	pub config: wgpu::SurfaceConfiguration,
	pub adapter: wgpu::Adapter,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,

	pub surface_error: Option<wgpu::SurfaceError>,

	window: Arc<Window>,
	pub(crate) forms: Vec<FormStorage>,
	pub(crate) shades: Vec<ShadeStorage>,
	pub(crate) textures: Vec<TextureStorage>,
	pub(crate) buffers: Vec<wgpu::Buffer>,
	pub(crate) samplers: Vec<wgpu::Sampler>,
	pub(crate) shapes: Vec<ShapeStorage>,
	pub(crate) effects: Vec<EffectStorage>,
	pub(crate) layers: Vec<LayerStorage>,
	pub(crate) bind_groups: Vec<BindGroupStorage>,
	pub(crate) bind_group_layouts: Vec<wgpu::BindGroupLayout>,
	pub(crate) pipelines: BTreeMap<Vec<u8>, PipelineStorage>,
	pub(crate) fullscreen_quad_shader: wgpu::ShaderModule,
}

pub(crate) struct PainterConfig {
	pub use_vsync: bool,
	pub features: Option<wgpu::Features>,
}

impl Painter {
	pub(crate) async fn new(window: Arc<Window>, painter_config: PainterConfig) -> Self {
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
			.request_device(&wgpu::DeviceDescriptor {
				label: None,
				required_features: painter_config.features.unwrap_or(wgpu::Features::empty()),
				// Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
				required_limits: wgpu::Limits::downlevel_webgl2_defaults()
					.using_resolution(adapter.limits()),
				memory_hints: wgpu::MemoryHints::MemoryUsage,
				trace: wgpu::Trace::Off,
			})
			.await
			.expect("Failed to create device");

		let surface_caps = surface.get_capabilities(&adapter);
		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_caps.formats[0],
			width: size.width,
			height: size.height,
			present_mode: if painter_config.use_vsync {
				wgpu::PresentMode::AutoVsync
			} else {
				wgpu::PresentMode::AutoNoVsync
			},
			alpha_mode: surface_caps.alpha_modes[0],
			view_formats: vec![],
			desired_maximum_frame_latency: 2,
		};

		surface.configure(&device, &config);

		let fullscreen_quad_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Fullscreen Quad Shader"),
			source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(FULL_SCREEN_QUAD)),
		});

		let mut painter = Self {
			surface,
			config,
			adapter,
			device,
			queue,
			surface_error: None,
			window: window.clone(),
			forms: Vec::with_capacity(8),
			shades: Vec::with_capacity(8),
			textures: Vec::with_capacity(8),
			buffers: Vec::with_capacity(32),
			samplers: Vec::with_capacity(8),
			shapes: Vec::with_capacity(8),
			effects: Vec::with_capacity(8),
			layers: Vec::with_capacity(8),
			bind_group_layouts: Vec::with_capacity(8),
			bind_groups: Vec::with_capacity(8),
			pipelines: BTreeMap::new(),
			fullscreen_quad_shader,
		};

		Sampler::create(&mut painter, SamplerProps::NEAREST);
		Sampler::create(&mut painter, SamplerProps::LINEAR);

		let layer_sampler_layout =
			BindGroupLayout::values(&mut painter, &[BINDING_SAMPLER_FRAG]).unwrap();
		let layer_texture_layout =
			BindGroupLayout::layers(&mut painter, &[BINDING_LAYER_FRAG]).unwrap();

		BindGroup::values_bind_groups(
			&mut painter,
			1,
			Some(layer_sampler_layout),
			&Vec::with_capacity(0),
			&Vec::with_capacity(0),
			&vec![(0, Sampler(0).binding())],
		);

		BindGroup::values_bind_groups(
			&mut painter,
			1,
			Some(layer_sampler_layout),
			&Vec::with_capacity(0),
			&Vec::with_capacity(0),
			&vec![(0, Sampler(1).binding())],
		);

		let fullscreen_quad_pipeline_layout =
			painter
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: None,
					bind_group_layouts: &[
						&painter.bind_group_layouts[layer_sampler_layout.0],
						&painter.bind_group_layouts[layer_texture_layout.0],
					],
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
				layer: None,
				shape: None,
				effect: None,
			},
		);

		painter
	}

	pub fn window(&self) -> &Arc<Window> {
		&self.window
	}

	// form helpers

	pub fn form<'a>(&mut self, buffer: impl Into<FormBuffer<'a>>) -> FormBuilder<'_, 'a> {
		FormBuilder::new(self).with_buffer(buffer)
	}

	pub fn form_builder(&mut self) -> FormBuilder<'_, '_> {
		FormBuilder::new(self)
	}

	pub fn form_update<'a>(&mut self, form: Form, buffer: impl Into<FormBuffer<'a>>) {
		form.update(self, buffer);
	}

	// shade helpers

	pub fn shade<Format: Into<AttribsFormat>>(
		&mut self,
		attributes: Format,
	) -> ShadeBuilder<'_, Format> {
		ShadeBuilder::new(self, attributes)
	}

	pub fn shade_effect(&mut self) -> ShadeEffectBuilder<'_> {
		ShadeEffectBuilder::new(self)
	}

	// texture helpers

	pub fn sampler(&mut self) -> SamplerBuilder<'_> {
		SamplerBuilder::new(self)
	}

	pub fn sampler_nearest(&self) -> Sampler {
		Sampler(0)
	}

	pub fn sampler_linear(&self) -> Sampler {
		Sampler(1)
	}

	// shape utils

	pub fn shape(&mut self, form: Form, shade: Shade) -> ShapeBuilder<'_> {
		ShapeBuilder::new(self, form, shade)
	}

	pub fn effect(&mut self, shade: Shade) -> EffectBuilder<'_> {
		EffectBuilder::new(self, shade)
	}

	// layer utils

	pub fn layer<'b>(&mut self) -> LayerBuilder<'_, 'b> {
		LayerBuilder::new(self)
	}

	pub fn single_effect_layer(&mut self, shade: Shade) -> SingleEffectLayerBuilder<'_> {
		SingleEffectLayerBuilder::new(self, shade)
	}

	// binding utils

	pub fn bind_buff<T: bytemuck::Pod>(&mut self, data: T) -> BindingBuffer<T> {
		BindingBuffer::new(self, data)
	}
	pub fn bind_mat3(&mut self) -> BindingBuffer<Mat3U> {
		self.bind_buff(Mat3U(Mat3A::IDENTITY))
	}
	pub fn bind_mat4(&mut self) -> BindingBuffer<Mat4> {
		self.bind_buff(Mat4::IDENTITY)
	}
	pub fn bind_vec2(&mut self) -> BindingBuffer<Vec2> {
		self.bind_buff(Vec2::ZERO)
	}
	pub fn bind_vec3(&mut self) -> BindingBuffer<Vec3U> {
		self.bind_buff(Vec3U(Vec3A::ZERO))
	}
	pub fn bind_vec4(&mut self) -> BindingBuffer<Vec4> {
		self.bind_buff(Vec4::ZERO)
	}
	pub fn bind_uvec2(&mut self) -> BindingBuffer<UVec2> {
		self.bind_buff(UVec2::ZERO)
	}
	pub fn bind_f32(&mut self) -> BindingBuffer<f32> {
		self.bind_buff(0.0f32)
	}
	pub fn bind_u32(&mut self) -> BindingBuffer<u32> {
		self.bind_buff(0u32)
	}
	pub fn bind_quat(&mut self) -> BindingBuffer<Quat> {
		self.bind_buff(Quat::IDENTITY)
	}

	pub fn bind_const_buff<T: bytemuck::Pod>(&mut self, data: T) -> ValueBinding {
		self.bind_buff(data).binding()
	}
	pub fn bind_const_mat3(&mut self, mat: Mat3) -> ValueBinding {
		let u = self.bind_mat3();
		u.update_mat3(self, mat);
		u.binding()
	}
	pub fn bind_const_mat4(&mut self, mat: Mat4) -> ValueBinding {
		self.bind_const_buff(mat)
	}
	pub fn bind_const_vec2(&mut self, vec: Vec2) -> ValueBinding {
		self.bind_const_buff(vec)
	}
	pub fn bind_const_vec3(&mut self, vec: Vec3) -> ValueBinding {
		let u = self.bind_vec3();
		u.update_vec3(self, vec);
		u.binding()
	}
	pub fn bind_const_vec4(&mut self, vec: Vec4) -> ValueBinding {
		self.bind_const_buff(vec)
	}
	pub fn bind_const_uvec2(&mut self, vec: UVec2) -> ValueBinding {
		self.bind_const_buff(vec)
	}
	pub fn bind_const_f32(&mut self, f: f32) -> ValueBinding {
		self.bind_const_buff(f)
	}
	pub fn bind_const_u32(&mut self, u: u32) -> ValueBinding {
		self.bind_const_buff(u)
	}
	pub fn bind_const_quat(&mut self, quat: Quat) -> ValueBinding {
		self.bind_const_buff(quat)
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

	pub(crate) fn get_shape_pipeline_key(&self, shape: Shape, layer: Layer) -> Vec<u8> {
		let l = &self.layers[layer.0];
		let sp = &self.shapes[shape.0];
		[sp.pipeline_key.as_slice(), l.pipeline_key.as_slice()].concat()
	}

	pub(crate) fn ensure_shape_pipeline<'a>(
		&'a mut self,
		pipeline_key: &Vec<u8>,
		shape: Shape,
		layer: Layer,
	) {
		if !self.pipelines.contains_key(pipeline_key) {
			let pipeline = PipelineStorage::create_shape_pipeline(self, shape, layer);
			self.pipelines.insert(pipeline_key.clone(), pipeline);
		}
	}

	pub(crate) fn get_effect_pipeline_key(&self, effect: Effect, layer: Layer) -> Vec<u8> {
		let layer_key = self.layers[layer.0].pipeline_key.as_slice();
		let effect_key = self.effects[effect.0].pipeline_key.as_slice();
		[effect_key, layer_key].concat()
	}

	pub(crate) fn ensure_effect_pipeline<'a>(
		&mut self,
		pipeline_key: &Vec<u8>,
		effect: Effect,
		layer: Layer,
	) {
		if !self.pipelines.contains_key(pipeline_key) {
			let pipeline = PipelineStorage::create_effect_pipeline(self, effect, layer);
			self.pipelines.insert(pipeline_key.to_vec(), pipeline);
		}
	}

	/// Renders a shape with optimized instance handling.
	///
	/// # Instance Rendering Rules
	/// Based on the lengths of bind_groups (value bindings) and layer_bind_group_data (layer bindings):
	///
	/// 1. **No instances** (both ≤ 1): Set bindings once, single draw call
	/// 2. **Only value bindings vary** (values > 1, layers ≤ 1):
	///    Set layer bindings once, iterate through value bindings
	/// 3. **Only layer bindings vary** (layers > 1, values ≤ 1):
	///    Set value bindings once, iterate through layer bindings
	/// 4. **Both vary** (both > 1):
	///    Iterate through all instances, setting both bind groups per draw
	///
	/// This respects the override hierarchy: Layer → Shape → Instance
	fn render_shape(&self, pass: &mut wgpu::RenderPass<'_>, shape_index: usize, layer: Layer) {
		let shape_data = &self.layers[layer.0].shapes[shape_index];
		let shape = shape_data.shape;
		let s = &self.shapes[shape.0];
		let f = &self.forms[s.form.0];

		let draw = |pass: &mut wgpu::RenderPass, binding: Option<BindGroup>| {
			if let Some(binding) = binding {
				pass.set_bind_group(0, &self.bind_groups[binding.0].bind_group, &[]);
			}

			for i in 0..f.currently_active_buffers {
				let b = &f.buffers[i];
				pass.set_vertex_buffer(0, b.vertex_buffer.slice(..b.vertex_buffer_current_size));
				if let Some(index_buffer) = &b.index_buffer
					&& b.index_count > 0
				{
					pass.set_index_buffer(
						index_buffer.slice(..b.index_buffer_current_size),
						wgpu::IndexFormat::Uint32,
					);
					pass.draw_indexed(0..b.index_count, 0, 0..1);
				} else {
					pass.draw(0..b.vertex_count, 0..1);
				}
			}
		};

		let pipeline_key = self.get_shape_pipeline_key(shape, layer);
		let pipeline = &self.pipelines[&pipeline_key];
		pass.set_pipeline(&pipeline.pipeline);

		// Get bindings from shape_data
		let bind_groups = &shape_data.bind_groups;

		// Use pre-computed rendering strategy
		match shape_data.rendering_strategy {
			InstanceRenderingStrategy::NoInstances => {
				// Case 1: No instances - both bindings ≤ 1
				// Set both bind groups once and do single draw
				if let Some(layer_bind_group_data) = &shape_data.layer_bind_group_data {
					let layer_bind_group = layer_bind_group_data.to_gpu_bind_group(self);
					pass.set_bind_group(1, &layer_bind_group, &[]);
				}

				let value_binding = bind_groups.first().copied();
				draw(pass, value_binding);
			}
			InstanceRenderingStrategy::ValueBindingsVary => {
				// Case 2: Only value bindings vary (values > 1, layers ≤ 1)
				// Set layer bindings once, iterate through value bindings
				if let Some(layer_bind_group_data) = &shape_data.layer_bind_group_data {
					let layer_bind_group = layer_bind_group_data.to_gpu_bind_group(self);
					pass.set_bind_group(1, &layer_bind_group, &[]);
				}

				for value_bg in bind_groups {
					draw(pass, Some(*value_bg));
				}
			}
			InstanceRenderingStrategy::LayerBindingsVary => {
				// Case 3: Only layer bindings vary (layers > 1, values ≤ 1)
				// Set value bindings once, iterate through layer bindings
				if let Some(value_bg) = bind_groups.first() {
					pass.set_bind_group(0, &self.bind_groups[value_bg.0].bind_group, &[]);
				}

				if let Some(layer_bind_group_data) = &shape_data.layer_bind_group_data {
					let layer_bind_groups = layer_bind_group_data.to_gpu_bind_groups(self);
					for layer_bg in layer_bind_groups {
						pass.set_bind_group(1, &layer_bg, &[]);
						draw(pass, None);
					}
				}
			}
			InstanceRenderingStrategy::BothBindingsVary => {
				// Case 4: Both bindings vary (both > 1)
				// Iterate through all instances, setting both bind groups per draw
				if let Some(layer_bind_group_data) = &shape_data.layer_bind_group_data {
					let layer_bind_groups = layer_bind_group_data.to_gpu_bind_groups(self);

					for (value_bg, layer_bg) in bind_groups.iter().zip(layer_bind_groups.iter()) {
						pass.set_bind_group(0, &self.bind_groups[value_bg.0].bind_group, &[]);
						pass.set_bind_group(1, layer_bg, &[]);
						draw(pass, None);
					}
				}
			}
		}
	}

	/// Helper to create a single layer bind group, optionally prepending the source binding.
	fn create_layer_bind_group(
		&self,
		layer_bind_group_data: &LayerBindGroupData,
		source_binding: Option<&LayerBinding>,
	) -> wgpu::BindGroup {
		if let Some(binding) = source_binding {
			layer_bind_group_data.to_gpu_bind_group_with_first(self, binding)
		} else {
			layer_bind_group_data.to_gpu_bind_group(self)
		}
	}

	/// Helper to create multiple layer bind groups, optionally prepending the source binding to each.
	fn create_layer_bind_groups(
		&self,
		layer_bind_group_data: &LayerBindGroupData,
		source_binding: Option<&LayerBinding>,
	) -> Vec<wgpu::BindGroup> {
		if let Some(binding) = source_binding {
			layer_bind_group_data.to_gpu_bind_groups_with_first(self, binding)
		} else {
			layer_bind_group_data.to_gpu_bind_groups(self)
		}
	}

	/// Renders an effect with optimized instance handling.
	///
	/// # Instance Rendering Rules
	/// Based on the lengths of bind_groups (value bindings) and layer_bind_group_data (layer bindings):
	///
	/// 1. **No instances** (both ≤ 1): Set bindings once, single draw call
	/// 2. **Only value bindings vary** (values > 1, layers ≤ 1):
	///    Set layer bindings once, iterate through value bindings
	/// 3. **Only layer bindings vary** (layers > 1, values ≤ 1):
	///    Set value bindings once, iterate through layer bindings
	/// 4. **Both vary** (both > 1):
	///    Iterate through all instances, setting both bind groups per draw
	///
	/// This respects the override hierarchy: Layer → Effect → Instance
	fn render_effect(&self, effect_index: usize, layer: Layer, skip_source: bool) {
		let l = &self.layers[layer.0];
		let effect_data = &l.effects[effect_index];
		let effect = effect_data.effect;
		let e = &self.effects[effect.0];

		let view = if let Some(mip_level) = e.dst_mip_level {
			l.current_target_texture()
				.view(self, &TexViewKey::AtMipLevel(mip_level))
		} else {
			l.current_target_texture().target_view(self)
		};

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

		{
			let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
					depth_slice: None,
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
			});

			let pipeline_key = self.get_effect_pipeline_key(effect, layer);
			let pipeline = &self.pipelines[&pipeline_key];
			pass.set_pipeline(&pipeline.pipeline);

			// Get bindings from effect_data
			let bind_groups = &effect_data.bind_groups;

			// Pre-compute source binding if needed (avoids duplicate computation in all cases)
			let source_binding = if !skip_source {
				Some(if let Some(src_mip_level) = e.src_mip_level {
					layer.binding_at_mip_level(src_mip_level)
				} else {
					layer.binding()
				})
			} else {
				None
			};

			// Use pre-computed rendering strategy
			match effect_data.rendering_strategy {
				InstanceRenderingStrategy::NoInstances => {
					// Case 1: No instances - both bindings ≤ 1
					// Set both bind groups once and do single draw
					if let Some(layer_bind_group_data) = &effect_data.layer_bind_group_data {
						let layer_bind_group = self.create_layer_bind_group(
							layer_bind_group_data,
							source_binding.as_ref(),
						);
						pass.set_bind_group(1, &layer_bind_group, &[]);
					}

					if let Some(value_bg) = bind_groups.first() {
						pass.set_bind_group(0, &self.bind_groups[value_bg.0].bind_group, &[]);
					}
					pass.draw(0..3, 0..1);
				}
				InstanceRenderingStrategy::ValueBindingsVary => {
					// Case 2: Only value bindings vary (values > 1, layers ≤ 1)
					// Set layer bindings once, iterate through value bindings
					if let Some(layer_bind_group_data) = &effect_data.layer_bind_group_data {
						let layer_bind_group = self.create_layer_bind_group(
							layer_bind_group_data,
							source_binding.as_ref(),
						);
						pass.set_bind_group(1, &layer_bind_group, &[]);
					}

					for value_bg in bind_groups {
						pass.set_bind_group(0, &self.bind_groups[value_bg.0].bind_group, &[]);
						pass.draw(0..3, 0..1);
					}
				}
				InstanceRenderingStrategy::LayerBindingsVary => {
					// Case 3: Only layer bindings vary (layers > 1, values ≤ 1)
					// Set value bindings once, iterate through layer bindings
					if let Some(value_bg) = bind_groups.first() {
						pass.set_bind_group(0, &self.bind_groups[value_bg.0].bind_group, &[]);
					}

					if let Some(layer_bind_group_data) = &effect_data.layer_bind_group_data {
						let layer_bind_groups = self.create_layer_bind_groups(
							layer_bind_group_data,
							source_binding.as_ref(),
						);
						for layer_bg in layer_bind_groups {
							pass.set_bind_group(1, &layer_bg, &[]);
							pass.draw(0..3, 0..1);
						}
					} else {
						pass.draw(0..3, 0..1);
					}
				}
				InstanceRenderingStrategy::BothBindingsVary => {
					// Case 4: Both bindings vary (both > 1)
					// Iterate through all instances, setting both bind groups per draw
					if let Some(layer_bind_group_data) = &effect_data.layer_bind_group_data {
						let layer_bind_groups = self.create_layer_bind_groups(
							layer_bind_group_data,
							source_binding.as_ref(),
						);

						for (value_bg, layer_bg) in bind_groups.iter().zip(layer_bind_groups.iter())
						{
							pass.set_bind_group(0, &self.bind_groups[value_bg.0].bind_group, &[]);
							pass.set_bind_group(1, layer_bg, &[]);
							pass.draw(0..3, 0..1);
						}
					}
				}
			}
		}

		self.queue.submit(Some(encoder.finish()));
	}

	pub fn paint(&mut self, layer: Layer) {
		let l = &self.layers[layer.0];
		let shapes_len = l.shapes.len();
		let effects_len = l.effects.len();
		let has_shapes = shapes_len > 0;

		if has_shapes {
			let color_attachments: Vec<Option<RenderPassColorAttachment<'_>>> =
				if !l.is_multi_target {
					let target_view = l.current_target_texture().target_view(self);
					let multisampled_texture = l.multisampled_textures.get(0);

					let view = multisampled_texture.map_or(target_view, |t| t.target_view(self));
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
						depth_slice: None,
					})]
				} else {
					l.target_textures
						.iter()
						.enumerate()
						.map(|(i, t)| {
							let target_view = t.target_view(self);
							let multisampled_texture = l.multisampled_textures.get(i);

							let view =
								multisampled_texture.map_or(target_view, |t| t.target_view(self));
							let resolve_target = multisampled_texture.map(|_| target_view);

							Some(wgpu::RenderPassColorAttachment {
								view,
								resolve_target,
								ops: wgpu::Operations {
									load: l.clear_color.map_or(wgpu::LoadOp::Load, |color| {
										wgpu::LoadOp::Clear(color)
									}),
									store: wgpu::StoreOp::Store,
								},
								depth_slice: None,
							})
						})
						.collect::<Vec<_>>()
				};

			let mut encoder = self
				.device
				.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

			{
				let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
					label: None,
					color_attachments: &color_attachments,
					depth_stencil_attachment: l.depth_texture.as_ref().map(|t| {
						wgpu::RenderPassDepthStencilAttachment {
							view: t.view(self, &TexViewKey::Default),
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
					self.render_shape(&mut pass, i, layer);
				}
			}

			self.queue.submit(Some(encoder.finish()));
		}

		if effects_len == 0 {
			l.current_target_texture().update_mips(self);
			return;
		}

		if has_shapes {
			self.layers[layer.0].swap_targets();
		}

		let mut update_mips = true;
		for i in 0..effects_len {
			let effect = self.layers[layer.0].effects[i].effect;
			let e = &self.effects[effect.0];

			let skip_source_tex = i == 0 && !(has_shapes || e.src_mip_level.is_some());
			self.render_effect(i, layer, skip_source_tex);

			if e.dst_mip_level.is_none() {
				self.layers[layer.0].swap_targets();
			} else {
				// If the effect has a mip target, we don't swap the targets.
				// Instead, we update the mips of the current target texture.
				update_mips = false;
			}
		}

		if update_mips {
			self.layers[layer.0]
				.current_source_texture()
				.update_mips(self);
		}
	}

	pub fn compose<I>(&mut self, layers: I)
	where
		I: IntoIterator<Item = Layer>,
	{
		for layer in layers {
			self.paint(layer);
		}
	}

	pub fn show(&mut self, layer: Layer) {
		let result = self.surface.get_current_texture();

		if result.is_err() {
			self.surface_error = Some(result.err().unwrap());
			return;
		}

		let frame = result.unwrap();

		let view = frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

		let pipeline = &self.pipelines[FULL_SCREEN_TEXTURE_PIPELINE];

		{
			let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: None,
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
						store: wgpu::StoreOp::Store,
					},
					depth_slice: None,
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
			});
			pass.set_pipeline(&pipeline.pipeline);
			pass.set_bind_group(0, &self.bind_groups[0].bind_group, &[]);
			pass.set_bind_group(
				1,
				&BindGroup::layer_gpu_bind_group(self, layer.binding()),
				&[],
			);
			pass.draw(0..3, 0..1);
		}

		self.queue.submit(Some(encoder.finish()));
		frame.present();
	}

	pub fn paint_and_show(&mut self, layer: Layer) {
		self.paint(layer);
		self.show(layer)
	}

	/// Initializes GPU pipelines for the layer and then renders it.
	/// Pipelines need to be initialized only once per layer, so this is a convenient method for the first paint.
	///
	/// Only needed if the layer was created outsite of CanvasApp::init, or if the layer should be rendered immediately after creation.
	/// Otherwise, all layers are initialized automatically after CanvasApp::init and before the first CanvasApp::render call.
	pub fn init_and_paint(&mut self, layer: Layer) {
		layer.init_gpu_pipelines(self);
		self.paint(layer);
	}

	#[cfg(not(target_arch = "wasm32"))]
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
					let pipeline = self.pipelines.remove(pipeline_key);
					if let Some(pipeline) = pipeline {
						let pipeline = pipeline.recreate(self);
						self.pipelines.insert(pipeline_key.clone(), pipeline);
					}
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
