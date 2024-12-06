use super::{form::Form, shade::Shade, uniform::Uniform, Painter};
use crate::utils::default;
use std::collections::HashMap;

pub(crate) struct SketchStorage {
	pub uniforms: HashMap<u32, Uniform>,
	pub instances: Vec<HashMap<u32, Uniform>>,
	pub form: Form,
	pub pipeline_key: String,
	pub depth_test: bool,
}

pub struct SketchProps {
	pub uniforms: HashMap<u32, Uniform>,
	pub instances: Vec<HashMap<u32, Uniform>>,
	pub cull_mode: Option<wgpu::Face>,
	pub depth_test: bool,
}

impl Default for SketchProps {
	fn default() -> Self {
		SketchProps {
			uniforms: HashMap::with_capacity(0),
			instances: Vec::with_capacity(0),
			cull_mode: Some(wgpu::Face::Back),
			depth_test: false,
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct Sketch(pub(crate) usize);

impl Sketch {
	pub fn new(painter: &mut Painter, form: Form, shade: Shade, props: &SketchProps) -> Self {
		let f = &painter.forms[form.0];
		let s = &painter.shades[shade.0];

		let pipeline_key = format!(
			"s{}-ft{}-ff{}-pd{}",
			shade.0, f.props.topology as u16, f.props.front_face as u16, props.depth_test
		);

		if !painter.pipelines.contains_key(&pipeline_key) {
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
					depth_stencil: if props.depth_test {
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
					multisample: wgpu::MultisampleState::default(),
					multiview: None,
					cache: None,
				});

			painter.pipelines.insert(pipeline_key.clone(), pipeline);
		}

		let sketch = SketchStorage {
			form,
			pipeline_key,
			uniforms: props.uniforms.clone(),
			instances: props.instances.clone(),
			depth_test: props.depth_test,
		};

		painter.sketches.push(sketch);

		Sketch(painter.sketches.len() - 1)
	}
}
