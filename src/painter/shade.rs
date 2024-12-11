use super::Painter;

pub(crate) struct ShadeStorage {
	pub vertex_shader: Option<wgpu::ShaderModule>,
	pub fragment_shader: wgpu::ShaderModule,
	pub attribs: AttribsFormat,
	pub pipeline_layout: wgpu::PipelineLayout,
}

pub struct ShadeProps<'a, Format: Into<AttribsFormat>> {
	pub vertex_shader: wgpu::ShaderModuleDescriptor<'a>,
	pub fragment_shader: wgpu::ShaderModuleDescriptor<'a>,
	pub vertex_format: Format,
	pub uniform_layout: &'a [&'a wgpu::BindGroupLayout],
}

pub struct ShadeEffectProps<'a> {
	pub shader: wgpu::ShaderModuleDescriptor<'a>,
	pub uniform_layout: &'a [&'a wgpu::BindGroupLayout],
}

pub struct AttribsFormat {
	pub stride: u64,
	pub attributes: Vec<wgpu::VertexAttribute>,
}

pub fn attrib(location: u32, format: wgpu::VertexFormat, offset: u64) -> wgpu::VertexAttribute {
	wgpu::VertexAttribute {
		shader_location: location,
		format,
		offset,
	}
}

impl Into<AttribsFormat> for &[wgpu::VertexFormat] {
	fn into(self) -> AttribsFormat {
		let mut stride = 0;
		let mut attributes = Vec::with_capacity(self.len());
		let mut location = 0;
		for format in self {
			attributes.push(attrib(location, *format, stride));
			stride += format.size();
			location += 1;
		}

		AttribsFormat { attributes, stride }
	}
}

impl Into<AttribsFormat> for Vec<wgpu::VertexFormat> {
	fn into(self) -> AttribsFormat {
		self.as_slice().into()
	}
}

impl Into<AttribsFormat> for wgpu::VertexFormat {
	fn into(self) -> AttribsFormat {
		AttribsFormat {
			attributes: vec![attrib(0, self, 0)],
			stride: self.size(),
		}
	}
}

#[derive(Clone, Copy)]
pub struct Shade(pub(crate) usize);

impl Shade {
	pub fn new<Format: Into<AttribsFormat>>(
		painter: &mut Painter,
		props: ShadeProps<Format>,
	) -> Self {
		let vertex_shader = painter.device.create_shader_module(props.vertex_shader);
		let fragment_shader = painter.device.create_shader_module(props.fragment_shader);

		let pipeline_layout =
			painter
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: None,
					bind_group_layouts: props.uniform_layout,
					push_constant_ranges: &[],
				});

		let format = props.vertex_format.into();

		let s = ShadeStorage {
			vertex_shader: Some(vertex_shader),
			fragment_shader,
			attribs: format,
			pipeline_layout,
		};

		let i = painter.shades.len();
		painter.shades.push(s);

		Shade(i)
	}

	pub fn new_effect(painter: &mut Painter, props: ShadeEffectProps) -> Self {
		let fragment_shader = painter.device.create_shader_module(props.shader);

		let pipeline_layout =
			painter
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: None,
					bind_group_layouts: props.uniform_layout,
					push_constant_ranges: &[],
				});

		let format = vec![].into();

		let s = ShadeStorage {
			vertex_shader: None,
			fragment_shader,
			attribs: format,
			pipeline_layout,
		};

		let i = painter.shades.len();
		painter.shades.push(s);

		Shade(i)
	}

	pub fn form_stride(&self, painter: &Painter) -> u64 {
		painter.shades[self.0].attribs.stride
	}
}
