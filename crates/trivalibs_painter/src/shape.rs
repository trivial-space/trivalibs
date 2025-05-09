use crate::{
	binding::Binding,
	form::Form,
	layer::Layer,
	shade::Shade,
	uniform::{InstanceUniforms, Uniform},
	Painter,
};

#[derive(Clone)]
pub(crate) struct ShapeStorage {
	pub form: Form,
	pub shade: Shade,
	pub uniforms: Vec<(u32, Uniform)>,
	pub effect_layers: Vec<(u32, Layer)>,
	pub instances: Vec<InstanceUniforms>,
	pub pipeline_key: Vec<u8>,
	pub cull_mode: Option<wgpu::Face>,
	pub blend_state: wgpu::BlendState,
	pub uniform_bindings: Vec<Binding>,
}

#[derive(Clone)]
pub struct ShapeProps {
	pub uniforms: Vec<(u32, Uniform)>,
	pub effect_layers: Vec<(u32, Layer)>,
	pub instances: Vec<InstanceUniforms>,
	pub cull_mode: Option<wgpu::Face>,
	pub blend_state: wgpu::BlendState,
}

impl Default for ShapeProps {
	fn default() -> Self {
		ShapeProps {
			uniforms: Vec::with_capacity(0),
			effect_layers: Vec::with_capacity(0),
			instances: Vec::with_capacity(0),
			cull_mode: Some(wgpu::Face::Back),
			blend_state: wgpu::BlendState::REPLACE,
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct Shape(pub(crate) usize);

impl Shape {
	pub fn new(painter: &mut Painter, form: Form, shade: Shade, props: ShapeProps) -> Self {
		let f = &painter.forms[form.0];

		let pipeline_key = vec![
			(shade.0 as u16).to_le_bytes().to_vec(),
			vec![
				f.props.topology as u8,
				f.props.front_face as u8,
				props.blend_state.alpha.dst_factor as u8,
				props.blend_state.alpha.src_factor as u8,
				props.blend_state.alpha.operation as u8,
				props.blend_state.color.dst_factor as u8,
				props.blend_state.color.src_factor as u8,
				props.blend_state.color.operation as u8,
				if let Some(cull_mode) = props.cull_mode {
					cull_mode as u8
				} else {
					0xff
				},
			],
		]
		.into_iter()
		.flatten()
		.collect();

		let shape = ShapeStorage {
			form,
			shade,
			pipeline_key,
			uniforms: props.uniforms,
			effect_layers: props.effect_layers,
			instances: props.instances,
			cull_mode: props.cull_mode,
			blend_state: props.blend_state,
			uniform_bindings: Vec::with_capacity(0),
		};

		painter.shapes.push(shape);

		Shape(painter.shapes.len() - 1)
	}

	pub(crate) fn prepare_uniforms(&self, painter: &mut Painter, layer: Layer) {
		let sp = &painter.shapes[self.0];
		let sd = &painter.shades[sp.shade.0];
		let data = &sp.uniforms.clone();
		let instances = &sp.instances.clone();
		let layer_data = &painter.layers[layer.0].uniforms.clone();

		let uniforms = Binding::uniforms(
			painter,
			sd.uniforms_length,
			sd.uniform_layout,
			data,
			instances,
			layer_data,
		);

		painter.shapes[self.0].uniform_bindings = uniforms;
	}
}

/// Builder for creating new [`Shape`]s with custom properties.
///
/// # Default values for [`ShapeProps`]:
/// - `cull_mode`: `Some(wgpu::Face::Back)`
/// - `blend_state`: `wgpu::BlendState::REPLACE`
///
/// # Example
/// ```
/// let shape = ShapeBuilder::new(painter, form, shade)
///     .with_uniforms(uniforms)
///     .with_instances(instances)
///     .create();
/// ```
pub struct ShapeBuilder<'a> {
	form: Form,
	shade: Shade,
	painter: &'a mut Painter,
	props: ShapeProps,
}

impl<'a> ShapeBuilder<'a> {
	pub fn new(painter: &'a mut Painter, form: Form, shade: Shade) -> Self {
		ShapeBuilder {
			form,
			shade,
			painter,
			props: ShapeProps::default(),
		}
	}

	pub fn create(self) -> Shape {
		Shape::new(self.painter, self.form, self.shade, self.props)
	}

	pub fn with_uniforms(mut self, uniforms: Vec<(u32, Uniform)>) -> Self {
		self.props.uniforms = uniforms;
		self
	}

	pub fn with_effect_layers(mut self, effect_layers: Vec<(u32, Layer)>) -> Self {
		self.props.effect_layers = effect_layers;
		self
	}

	pub fn with_instances(mut self, instances: Vec<InstanceUniforms>) -> Self {
		self.props.instances = instances;
		self
	}

	pub fn with_cull_mode(mut self, cull_mode: Option<wgpu::Face>) -> Self {
		self.props.cull_mode = cull_mode;
		self
	}

	pub fn with_blend_state(mut self, blend_state: wgpu::BlendState) -> Self {
		self.props.blend_state = blend_state;
		self
	}
}
