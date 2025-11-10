use crate::{
	Painter,
	bind_group::{BindGroup, LayerBindGroupData},
	binding::{InstanceBinding, LayerBinding, ValueBinding},
	layer::Layer,
	shade::Shade,
};

#[derive(Clone)]
pub struct EffectProps {
	pub bindings: Vec<(u32, ValueBinding)>,
	pub layers: Vec<(u32, LayerBinding)>,
	pub instances: Vec<InstanceBinding>,
	pub blend_state: wgpu::BlendState,
	pub dst_mip_level: Option<u32>,
	pub src_mip_level: Option<u32>,
}

impl Default for EffectProps {
	fn default() -> Self {
		EffectProps {
			bindings: Vec::with_capacity(0),
			layers: Vec::with_capacity(0),
			instances: Vec::with_capacity(0),
			blend_state: wgpu::BlendState::REPLACE,
			dst_mip_level: None,
			src_mip_level: None,
		}
	}
}

pub(crate) struct EffectStorage {
	pub shade: Shade,
	pub bindings: Vec<(u32, ValueBinding)>,
	pub layers: Vec<(u32, LayerBinding)>,
	pub instances: Vec<InstanceBinding>,
	pub pipeline_key: Vec<u8>,
	pub blend_state: wgpu::BlendState,
	pub bind_groups: Vec<BindGroup>,
	pub layer_bind_group_data: Option<LayerBindGroupData>,
	pub dst_mip_level: Option<u32>,
	pub src_mip_level: Option<u32>,
}

#[derive(Clone, Copy, Debug)]
pub struct Effect(pub(crate) usize);

impl Effect {
	pub fn new(painter: &mut Painter, shade: Shade, props: EffectProps) -> Self {
		let pipeline_key = vec![
			(shade.0 as u16).to_le_bytes().to_vec(),
			vec![
				props.blend_state.alpha.dst_factor as u8,
				props.blend_state.alpha.src_factor as u8,
				props.blend_state.alpha.operation as u8,
				props.blend_state.color.dst_factor as u8,
				props.blend_state.color.src_factor as u8,
				props.blend_state.color.operation as u8,
			],
		]
		.into_iter()
		.flatten()
		.collect();

		let effect = EffectStorage {
			bindings: props.bindings,
			layers: props.layers,
			instances: props.instances,
			shade,
			pipeline_key,
			blend_state: props.blend_state,
			bind_groups: Vec::with_capacity(0),
			layer_bind_group_data: None,
			dst_mip_level: props.dst_mip_level,
			src_mip_level: props.src_mip_level,
		};

		painter.effects.push(effect);

		Self(painter.effects.len() - 1)
	}

	pub(crate) fn prepare_bindings(&self, painter: &mut Painter, layer: Layer) {
		let e = &painter.effects[self.0];
		let s = &painter.shades[e.shade.0];
		let l = &painter.layers[layer.0];

		let value_bindings = &e.bindings.clone();
		let instance_bindings = &e.instances.clone();
		let layer_bindings = &l.bindings.clone();

		let layer_bind_group_data = LayerBindGroupData::from_bindings(
			s.layer_bindings_length,
			s.layers_layout,
			&e.layers.clone(),
			&l.layers.clone(),
		);

		let bind_groups = BindGroup::values_bind_groups(
			painter,
			s.value_bindings_length,
			s.binding_layout,
			value_bindings,
			instance_bindings,
			layer_bindings,
		);

		painter.effects[self.0].bind_groups = bind_groups;
		painter.effects[self.0].layer_bind_group_data = layer_bind_group_data;
	}

	pub fn has_mip_target(&self, painter: &Painter) -> bool {
		let e = &painter.effects[self.0];
		e.dst_mip_level.is_some()
	}
	pub fn has_mip_source(&self, painter: &Painter) -> bool {
		let e = &painter.effects[self.0];
		e.src_mip_level.is_some()
	}
}

pub struct EffectBuilder<'a> {
	shade: Shade,
	props: EffectProps,
	painter: &'a mut Painter,
}

impl<'a> EffectBuilder<'a> {
	pub fn new(painter: &'a mut Painter, shade: Shade) -> Self {
		EffectBuilder {
			shade,
			painter,
			props: EffectProps::default(),
		}
	}

	pub fn create(self) -> Effect {
		Effect::new(self.painter, self.shade, self.props)
	}

	pub fn with_bindings(mut self, bindings: Vec<(u32, ValueBinding)>) -> Self {
		self.props.bindings = bindings;
		self
	}

	pub fn with_layers(mut self, layers: Vec<(u32, LayerBinding)>) -> Self {
		self.props.layers = layers;
		self
	}

	/// Repeatedly render this effect multiple times with different bindings into the same target without target swapping.
	/// This is useful for example for deferred lighting, where each light is rendered with custom blend state on top of the last.
	pub fn with_instances(mut self, instances: Vec<InstanceBinding>) -> Self {
		self.props.instances = instances;
		self
	}

	pub fn with_blend_state(mut self, blend_state: wgpu::BlendState) -> Self {
		self.props.blend_state = blend_state;
		self
	}

	pub fn with_mip_target(mut self, dst_mip_level: u32) -> Self {
		self.props.dst_mip_level = Some(dst_mip_level);
		self
	}

	pub fn with_mip_source(mut self, src_mip_level: u32) -> Self {
		self.props.src_mip_level = Some(src_mip_level);
		self
	}
}
