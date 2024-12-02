use std::collections::HashMap;

use crate::utils::default;

use super::{form::Form, shade::Shade, uniform::Uniform, Painter};

pub(crate) struct SketchStorage {
	pub uniforms: HashMap<u32, Uniform>,
	pub instances: Vec<HashMap<u32, Uniform>>,
	pub form: Form,
	pub pipeline: wgpu::RenderPipeline,
}

pub struct SketchProps {
	pub uniforms: HashMap<u32, Uniform>,
	pub instances: Vec<HashMap<u32, Uniform>>,
	pub cull_mode: Option<wgpu::Face>,
	/// TODO: Implement antialiasing
	pub antialias: bool,
}

impl Default for SketchProps {
	fn default() -> Self {
		SketchProps {
			uniforms: HashMap::with_capacity(0),
			instances: Vec::with_capacity(0),
			cull_mode: Some(wgpu::Face::Back),
			antialias: false,
		}
	}
}

pub struct Sketch(pub(crate) usize);

impl Sketch {
	pub fn new(painter: &mut Painter, form: Form, shade: Shade, props: &SketchProps) -> Self {
		let f = &painter.forms[form.0];
		let s = &painter.shades[shade.0];

		// TODO: Store pipelines per shade and props
		// Sketches with the same configuration should share the same pipeline
		let pipeline = painter
			.device
			.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				label: None,
				layout: Some(&s.pipeline_layout),
				vertex: wgpu::VertexState {
					module: &s.vertex_shader,
					entry_point: None,
					buffers: &[wgpu::VertexBufferLayout {
						array_stride: s.attribs.stride,
						step_mode: wgpu::VertexStepMode::Vertex,
						attributes: &s.attribs.attributes,
					}],
					compilation_options: default(),
				},
				fragment: Some(wgpu::FragmentState {
					module: &s.fragment_shader,
					entry_point: None,
					targets: &[Some(wgpu::ColorTargetState {
						format: painter.config.format,
						blend: Some(wgpu::BlendState::REPLACE),
						write_mask: wgpu::ColorWrites::ALL,
					})],
					compilation_options: default(),
				}),
				primitive: wgpu::PrimitiveState {
					topology: f.props.topology,
					strip_index_format: None,
					front_face: f.props.front_face,
					cull_mode: props.cull_mode,
					// cull_mode: None,
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

		let sketch = SketchStorage {
			form,
			pipeline,
			uniforms: props.uniforms.clone(),
			instances: props.instances.clone(),
		};

		painter.sketches.push(sketch);

		Sketch(painter.sketches.len() - 1)
	}
}
