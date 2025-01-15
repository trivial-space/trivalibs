use trivalibs_core::utils::default;
use wgpu::{util::make_spirv, ColorTargetState};

use crate::{effect::Effect, layer::Layer, shape::Shape, Painter};

pub(crate) struct PipelineStorage {
	pub pipeline: wgpu::RenderPipeline,
	pub layer: Option<Layer>,
	pub shape: Option<Shape>,
	pub effect: Option<Effect>,
}

impl PipelineStorage {
	pub(crate) fn create_shape_pipeline(painter: &Painter, shape: Shape, layer: Layer) -> Self {
		let l = &painter.layers[layer.0];
		let sp = &painter.shapes[shape.0];
		let sd = &painter.shades[sp.shade.0];
		let f = &painter.forms[sp.form.0];

		let targets: Vec<Option<ColorTargetState>> = l
			.formats
			.iter()
			.map(|f| {
				Some(wgpu::ColorTargetState {
					format: *f,
					blend: Some(sp.blend_state),
					write_mask: wgpu::ColorWrites::ALL,
				})
			})
			.collect::<Vec<_>>();

		let vertex_shader = painter
			.device
			.create_shader_module(wgpu::ShaderModuleDescriptor {
				label: None,
				source: make_spirv(&sd.vertex_bytes.as_ref().unwrap()),
			});

		let fragment_shader = painter
			.device
			.create_shader_module(wgpu::ShaderModuleDescriptor {
				label: None,
				source: make_spirv(&sd.fragment_bytes.as_ref().unwrap()),
			});

		let pipeline = painter
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
				depth_stencil: if l.depth_texture.is_some() {
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
					count: if l.multisampled_textures.is_empty() {
						1
					} else {
						4
					},
					mask: !0,
					alpha_to_coverage_enabled: false,
				},
				multiview: None,
				cache: None,
			});

		PipelineStorage {
			pipeline,
			effect: None,
			layer: Some(layer),
			shape: Some(shape),
		}
	}

	pub(crate) fn create_effect_pipeline(painter: &Painter, effect: Effect, layer: Layer) -> Self {
		let e = &painter.effects[effect.0];
		let s = &painter.shades[e.shade.0];
		let l = &painter.layers[layer.0];

		let fragment_shader = painter
			.device
			.create_shader_module(wgpu::ShaderModuleDescriptor {
				label: None,
				source: make_spirv(&s.fragment_bytes.as_ref().unwrap()),
			});

		let targets: Vec<Option<ColorTargetState>> = l
			.formats
			.iter()
			.map(|f| {
				Some(wgpu::ColorTargetState {
					format: *f,
					blend: Some(e.blend_state),
					write_mask: wgpu::ColorWrites::ALL,
				})
			})
			.collect::<Vec<_>>();

		let pipeline = painter
			.device
			.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				label: None,
				layout: Some(&s.pipeline_layout),
				vertex: wgpu::VertexState {
					module: &painter.fullscreen_quad_shader,
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

		PipelineStorage {
			pipeline,
			layer: Some(layer),
			effect: Some(effect),
			shape: None,
		}
	}

	pub(crate) fn recreate(self, painter: &Painter) -> Self {
		if let Some(layer) = self.layer {
			if let Some(effect) = self.effect {
				return Self::create_effect_pipeline(painter, effect, layer);
			} else if let Some(shape) = self.shape {
				return Self::create_shape_pipeline(painter, shape, layer);
			}
		}

		self
	}
}
