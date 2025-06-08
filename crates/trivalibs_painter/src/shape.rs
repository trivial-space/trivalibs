use crate::{
	bind_group::{BindGroup, LayerBindGroupData},
	binding::{InstanceBinding, LayerBinding, ValueBinding},
	form::Form,
	layer::Layer,
	shade::Shade,
	Painter,
};

#[derive(Clone)]
pub(crate) struct ShapeStorage {
	pub form: Form,
	pub shade: Shade,
	pub bindings: Vec<(u32, ValueBinding)>,
	pub layers: Vec<(u32, LayerBinding)>,
	pub instances: Vec<InstanceBinding>,
	pub pipeline_key: Vec<u8>,
	pub cull_mode: Option<wgpu::Face>,
	pub blend_state: wgpu::BlendState,
	pub bind_groups: Vec<BindGroup>,
	pub layer_bind_group_data: Option<LayerBindGroupData>,
}

#[derive(Clone)]
pub struct ShapeProps {
	pub bindings: Vec<(u32, ValueBinding)>,
	pub layers: Vec<(u32, LayerBinding)>,
	pub instances: Vec<InstanceBinding>,
	pub cull_mode: Option<wgpu::Face>,
	pub blend_state: wgpu::BlendState,
}

impl Default for ShapeProps {
	fn default() -> Self {
		ShapeProps {
			bindings: Vec::with_capacity(0),
			layers: Vec::with_capacity(0),
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
			bindings: props.bindings,
			layers: props.layers,
			instances: props.instances,
			cull_mode: props.cull_mode,
			blend_state: props.blend_state,
			bind_groups: Vec::with_capacity(0),
			layer_bind_group_data: None,
		};

		painter.shapes.push(shape);

		Shape(painter.shapes.len() - 1)
	}

	pub(crate) fn prepare_bindings(&self, painter: &mut Painter, layer: Layer) {
		let sp = &painter.shapes[self.0];
		let sd = &painter.shades[sp.shade.0];
		let l = &painter.layers[layer.0];

		let value_bindings = &sp.bindings.clone();
		let layer_bindings = &l.bindings.clone();
		let instances = &sp.instances.clone();

		let layer_bind_group_data = LayerBindGroupData::from_bindings(
			sd.layer_bindings_length,
			sd.layers_layout,
			&sp.layers.clone(),
			&l.layers.clone(),
		);

		let bind_groups = BindGroup::values_bind_groups(
			painter,
			sd.value_bindings_length,
			sd.binding_layout,
			value_bindings,
			instances,
			layer_bindings,
		);

		painter.shapes[self.0].bind_groups = bind_groups;
		painter.shapes[self.0].layer_bind_group_data = layer_bind_group_data;
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
///     .with_bindings(bindings)
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

	pub fn with_bindings(mut self, bindings: Vec<(u32, ValueBinding)>) -> Self {
		self.props.bindings = bindings;
		self
	}

	pub fn with_layers(mut self, layers: Vec<(u32, LayerBinding)>) -> Self {
		self.props.layers = layers;
		self
	}

	pub fn with_instances(mut self, instances: Vec<InstanceBinding>) -> Self {
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
