use super::{painter::UniformType, Painter};
use std::fs;

pub(crate) struct ShadeStorage {
	pub vertex_path: Option<String>,
	pub vertex_bytes: Option<Vec<u8>>,
	pub fragment_path: Option<String>,
	pub fragment_bytes: Option<Vec<u8>>,
	pub attribs: AttribsFormat,
	pub pipeline_layout: wgpu::PipelineLayout,
}

pub struct ShadeProps<'a, Format: Into<AttribsFormat>, UType: UniformType> {
	pub vertex_format: Format,
	pub uniform_types: &'a [&'a UType],
}

pub struct ShadeEffectProps<'a, UType: UniformType> {
	pub uniform_types: &'a [&'a UType],
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

impl Into<AttribsFormat> for &[wgpu::VertexFormat; 1] {
	fn into(self) -> AttribsFormat {
		self.as_slice().into()
	}
}
impl Into<AttribsFormat> for &[wgpu::VertexFormat; 2] {
	fn into(self) -> AttribsFormat {
		self.as_slice().into()
	}
}
impl Into<AttribsFormat> for &[wgpu::VertexFormat; 3] {
	fn into(self) -> AttribsFormat {
		self.as_slice().into()
	}
}
impl Into<AttribsFormat> for &[wgpu::VertexFormat; 4] {
	fn into(self) -> AttribsFormat {
		self.as_slice().into()
	}
}
impl Into<AttribsFormat> for &[wgpu::VertexFormat; 5] {
	fn into(self) -> AttribsFormat {
		self.as_slice().into()
	}
}
impl Into<AttribsFormat> for &[wgpu::VertexFormat; 6] {
	fn into(self) -> AttribsFormat {
		self.as_slice().into()
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
	pub fn new<Format: Into<AttribsFormat>, UType: UniformType>(
		painter: &mut Painter,
		props: ShadeProps<Format, UType>,
	) -> Self {
		let pipeline_layout =
			painter
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: None,
					bind_group_layouts: props
						.uniform_types
						.iter()
						.map(|t| t.layout())
						.collect::<Vec<_>>()
						.as_slice(),
					push_constant_ranges: &[],
				});

		let format = props.vertex_format.into();

		let s = ShadeStorage {
			vertex_path: None,
			vertex_bytes: None,
			fragment_path: None,
			fragment_bytes: None,
			attribs: format,
			pipeline_layout,
		};

		let i = painter.shades.len();
		painter.shades.push(s);

		Shade(i)
	}

	pub fn new_effect<UType: UniformType>(
		painter: &mut Painter,
		props: ShadeEffectProps<UType>,
	) -> Self {
		let pipeline_layout =
			painter
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: None,
					bind_group_layouts: props
						.uniform_types
						.iter()
						.map(|t| t.layout())
						.collect::<Vec<_>>()
						.as_slice(),
					push_constant_ranges: &[],
				});

		let format = vec![].into();

		let s = ShadeStorage {
			vertex_path: None,
			vertex_bytes: None,
			fragment_path: None,
			fragment_bytes: None,
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

	pub fn set_vertex_bytes(&self, painter: &mut Painter, bytes: Vec<u8>) {
		painter.shades[self.0].vertex_bytes = Some(bytes);
	}

	pub(crate) fn load_vertex_from_path(&self, painter: &mut Painter) {
		if let Some(shader_path) = &painter.shades[self.0].vertex_path {
			let bytes = fs::read(shader_path).expect("Failed to read vertex shader file");
			self.set_vertex_bytes(painter, bytes);
		}
	}

	pub fn set_vertex_path(&self, painter: &mut Painter, path: &str) {
		let path = get_full_path(path);
		painter.shades[self.0].vertex_path = Some(path);
		self.load_vertex_from_path(painter);
	}

	pub fn set_fragment_bytes(&self, painter: &mut Painter, bytes: Vec<u8>) {
		painter.shades[self.0].fragment_bytes = Some(bytes);
	}

	pub(crate) fn load_fragment_from_path(&self, painter: &mut Painter) {
		if let Some(shader_path) = &painter.shades[self.0].fragment_path {
			let bytes = fs::read(shader_path).expect("Failed to read fragment shader file");
			self.set_fragment_bytes(painter, bytes);
		}
	}

	pub fn set_fragment_path(&self, painter: &mut Painter, path: &str) {
		let path = get_full_path(path);
		painter.shades[self.0].fragment_path = Some(path);
		self.load_fragment_from_path(painter);
	}
}

fn get_full_path(path: &str) -> String {
	let current_file = file!();
	let current_dir = std::path::Path::new(current_file).parent().unwrap();
	let full_path = current_dir.join(path);
	let full_path = std::fs::canonicalize(full_path).unwrap();
	full_path.to_str().unwrap().to_string()
}