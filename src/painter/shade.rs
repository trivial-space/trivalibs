use crate::utils::default;

use super::Painter;

pub(crate) struct ShadeStorage {
	pub pipeline: wgpu::RenderPipeline,
}

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

#[derive(Clone, Copy)]
pub struct Shade(pub(crate) usize);

impl Shade {
	pub fn new<Format: Into<FormFormat>>(painter: &mut Painter, props: ShadeProps<Format>) -> Self {
		let pipeline_layout =
			painter
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: None,
					bind_group_layouts: props.uniform_layout,
					push_constant_ranges: &[],
				});

		let format = props.vertex_format.into();

		let vertex_shader = painter.device.create_shader_module(props.vertex_shader);
		let fragment_shader = painter.device.create_shader_module(props.fragment_shader);

		let pipeline = painter
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
						format: painter.config.format,
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

		let s = ShadeStorage { pipeline };

		let i = painter.shades.len();
		painter.shades.push(s);

		Shade(i)
	}
}
