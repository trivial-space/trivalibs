use super::Painter;
use crate::uniform::UniformType;
use std::fs;

pub(crate) struct ShadeStorage {
	pub vertex_path: Option<String>,
	pub vertex_bytes: Option<Vec<u8>>,
	pub fragment_path: Option<String>,
	pub fragment_bytes: Option<Vec<u8>>,
	pub attribs: AttribsFormat,
	pub pipeline_layout: wgpu::PipelineLayout,
}

pub struct ShadeProps<'a, Format: Into<AttribsFormat>> {
	pub vertex_format: Format,
	pub uniform_types: &'a [UniformType],
}

pub struct ShadeEffectProps<'a> {
	pub uniform_types: &'a [UniformType],
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
	pub fn new<Format: Into<AttribsFormat>>(
		painter: &mut Painter,
		props: ShadeProps<Format>,
	) -> Self {
		let pipeline_layout =
			painter
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: None,
					bind_group_layouts: props
						.uniform_types
						.iter()
						.map(|t| &painter.uniform_types[t.0].layout)
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

	pub fn new_effect(painter: &mut Painter, props: ShadeEffectProps) -> Self {
		let pipeline_layout =
			painter
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: None,
					bind_group_layouts: props
						.uniform_types
						.iter()
						.map(|t| &painter.uniform_types[t.0].layout)
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
		painter.shades[self.0].vertex_path = Some(path.to_string());
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
		painter.shades[self.0].fragment_path = Some(path.to_string());
		self.load_fragment_from_path(painter);
	}
}

#[macro_export]
macro_rules! load_fragment_shader {
	($shade:expr, $painter:expr, $path:expr) => {
		#[cfg(debug_assertions)]
		{
			let current_file = file!();
			let current_dir = std::path::Path::new(current_file).parent().unwrap();
			let full_path = current_dir.join($path);
			let full_path = std::fs::canonicalize(full_path).unwrap();
			let full_path = full_path.to_str().unwrap();
			println!("loading shader: {:?}", full_path);
			$shade.set_fragment_path($painter, full_path);
		}

		#[cfg(not(debug_assertions))]
		$shade.set_fragment_bytes($painter, include_bytes!($path).to_vec());
	};
}

#[macro_export]
macro_rules! load_vertex_shader {
	($shade:expr, $painter:expr, $path:expr) => {
		#[cfg(debug_assertions)]
		{
			let current_file = file!();
			let current_dir = std::path::Path::new(current_file).parent().unwrap();
			let full_path = current_dir.join($path);
			let full_path = std::fs::canonicalize(full_path).unwrap();
			let full_path = full_path.to_str().unwrap();
			println!("loading shader: {:?}", full_path);
			$shade.set_vertex_path($painter, full_path);
		}

		#[cfg(not(debug_assertions))]
		$shade.set_vertex_bytes($painter, include_bytes!($path).to_vec());
	};
}
