use crate::{
	bind_group::BindGroupLayout,
	binding::{BindingLayout, LayerLayout},
	prelude::BINDING_LAYER_FRAG,
	Painter,
};
use std::fs;

pub(crate) struct ShadeStorage {
	pub vertex_path: Option<String>,
	pub vertex_bytes: Option<Vec<u8>>,
	pub fragment_path: Option<String>,
	pub fragment_bytes: Option<Vec<u8>>,
	pub attribs: AttribsFormat,
	pub pipeline_layout: wgpu::PipelineLayout,
	pub binding_layout: Option<BindGroupLayout>,
	pub layers_layout: Option<BindGroupLayout>,
	pub value_bindings_length: usize,
	pub layer_bindings_length: usize,
}

pub struct ShadeProps<'a, Format: Into<AttribsFormat>> {
	pub attributes: Format,
	pub bindings: &'a [BindingLayout],
	pub layers: &'a [LayerLayout],
}

fn layouts_from_props(
	painter: &mut Painter,
	bindings: &[BindingLayout],
	layers: &[LayerLayout],
) -> (
	wgpu::PipelineLayout,
	Option<BindGroupLayout>,
	Option<BindGroupLayout>,
) {
	let bindings_layout = BindGroupLayout::values(painter, bindings);

	let layer_layout = BindGroupLayout::layers(painter, layers);

	let mut layouts = vec![];

	if let Some(l) = &bindings_layout {
		layouts.push(&painter.bind_group_layouts[l.0]);
	}

	if let Some(l) = &layer_layout {
		layouts.push(&painter.bind_group_layouts[l.0]);
	}

	let pipeline_layout = painter
		.device
		.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: layouts.as_slice(),
			push_constant_ranges: &[],
		});

	(pipeline_layout, bindings_layout, layer_layout)
}

impl Default for ShadeProps<'_, AttribsFormat> {
	fn default() -> Self {
		Self {
			attributes: AttribsFormat {
				attributes: vec![],
				stride: 0,
			},
			bindings: &[],
			layers: &[],
		}
	}
}

pub struct ShadeEffectProps<'a> {
	pub bindings: &'a [BindingLayout],
	pub layers: &'a [LayerLayout],
}

impl Default for ShadeEffectProps<'_> {
	fn default() -> Self {
		Self {
			bindings: &[],
			layers: &[],
		}
	}
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
		let format = props.attributes.into();

		let (pipeline_layout, binding_layout, layers_layout) =
			layouts_from_props(painter, props.bindings, props.layers);

		let s = ShadeStorage {
			vertex_path: None,
			vertex_bytes: None,
			fragment_path: None,
			fragment_bytes: None,
			attribs: format,
			pipeline_layout,
			binding_layout,
			layers_layout,
			value_bindings_length: props.bindings.len(),
			layer_bindings_length: props.layers.len(),
		};

		let i = painter.shades.len();
		painter.shades.push(s);

		Shade(i)
	}

	pub fn new_effect(painter: &mut Painter, props: ShadeEffectProps) -> Self {
		let (pipeline_layout, binding_layout, layers_layout) =
			layouts_from_props(painter, props.bindings, props.layers);

		let format = vec![].into();

		let s = ShadeStorage {
			vertex_path: None,
			vertex_bytes: None,
			fragment_path: None,
			fragment_bytes: None,
			attribs: format,
			pipeline_layout,
			binding_layout,
			layers_layout,
			value_bindings_length: props.bindings.len(),
			layer_bindings_length: props.layers.len(),
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

pub struct ShadeBuilder<'a, 'b, Format>
where
	Format: Into<AttribsFormat>,
{
	props: ShadeProps<'a, Format>,
	painter: &'b mut Painter,
}

impl<'a, 'b, Format> ShadeBuilder<'a, 'b, Format>
where
	Format: Into<AttribsFormat>,
{
	pub fn new(painter: &'b mut Painter, attributes: Format) -> Self {
		ShadeBuilder {
			props: ShadeProps {
				attributes,
				bindings: &[],
				layers: &[],
			},
			painter,
		}
	}

	pub fn create(self) -> Shade {
		Shade::new(self.painter, self.props)
	}

	pub fn with_bindings(mut self, bindings: &'a [BindingLayout]) -> Self {
		self.props.bindings = bindings;
		self
	}

	pub fn with_layers(mut self, layers: &'a [LayerLayout]) -> Self {
		self.props.layers = layers;
		self
	}
}

pub struct ShadeEffectBuilder<'a, 'b> {
	props: ShadeEffectProps<'a>,
	painter: &'b mut Painter,
}

impl<'a, 'b> ShadeEffectBuilder<'a, 'b> {
	pub fn new(painter: &'b mut Painter) -> Self {
		ShadeEffectBuilder {
			props: ShadeEffectProps {
				bindings: &[],
				layers: &[],
			},
			painter,
		}
	}

	pub fn create(self) -> Shade {
		Shade::new_effect(self.painter, self.props)
	}

	pub fn with_bindings(mut self, bindings: &'a [BindingLayout]) -> Self {
		self.props.bindings = bindings;
		self
	}

	pub fn with_layers(mut self, layers: &'a [LayerLayout]) -> Self {
		self.props.layers = layers;
		self
	}

	pub fn with_layer(mut self) -> Self {
		self.props.layers = &[BINDING_LAYER_FRAG];
		self
	}
}

#[macro_export]
macro_rules! load_fragment_shader {
	($shade:expr, $painter:expr, $path:expr) => {
		#[cfg(debug_assertions)]
		{
			let current_file = file!();
			let current_dir = std::path::Path::new(current_file).parent().unwrap();
			println!("try loading shader: {:?}, {:?}", current_dir, $path);
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
