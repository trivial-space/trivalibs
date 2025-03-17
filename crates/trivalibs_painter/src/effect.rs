use crate::{
	binding::Binding,
	layer::Layer,
	shade::Shade,
	uniform::{InstanceUniforms, Uniform},
	Painter,
};

pub(crate) struct EffectStorage {
	pub shade: Shade,
	pub uniforms: Vec<(u32, Uniform)>,
	pub effect_layer_uniforms: Vec<(u32, Layer)>,
	pub instances: Vec<InstanceUniforms>,
	pub pipeline_key: Vec<u8>,
	pub blend_state: wgpu::BlendState,
	pub uniform_binding_index: u32,
	pub uniform_bindings: Vec<Binding>,
}

#[derive(Clone)]
pub struct EffectProps {
	pub uniforms: Vec<(u32, Uniform)>,
	pub effect_layer_uniforms: Vec<(u32, Layer)>,
	pub instances: Vec<InstanceUniforms>,
	pub blend_state: wgpu::BlendState,
}

impl Default for EffectProps {
	fn default() -> Self {
		EffectProps {
			uniforms: Vec::with_capacity(0),
			effect_layer_uniforms: Vec::with_capacity(0),
			instances: Vec::with_capacity(0),
			blend_state: wgpu::BlendState::REPLACE,
		}
	}
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

		let s = &painter.shades[shade.0];

		let effect = EffectStorage {
			uniforms: props.uniforms,
			effect_layer_uniforms: props.effect_layer_uniforms,
			instances: props.instances,
			shade,
			pipeline_key,
			blend_state: props.blend_state,
			uniform_binding_index: s.layer_layouts.len() as u32,
			uniform_bindings: Vec::with_capacity(0),
		};

		painter.effects.push(effect);

		Self(painter.effects.len() - 1)
	}

	pub(crate) fn prepare_uniforms(&self, painter: &mut Painter, layer: Layer) {
		let e = &painter.effects[self.0];
		let s = &painter.shades[e.shade.0];
		let l = &painter.layers[layer.0];

		let data = &e.uniforms.clone();
		let instances = &e.instances.clone();
		let layer_data = &l.uniforms.clone();
		let uniforms = Binding::uniforms(
			painter,
			s.uniforms_length,
			s.uniform_layout,
			data,
			instances,
			layer_data,
		);

		painter.effects[self.0].uniform_bindings = uniforms;
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

	pub fn with_uniforms(mut self, uniforms: Vec<(u32, Uniform)>) -> Self {
		self.props.uniforms = uniforms;
		self
	}

	pub fn with_effect_layers(mut self, effect_layers: Vec<(u32, Layer)>) -> Self {
		self.props.effect_layer_uniforms = effect_layers;
		self
	}

	/// Repeatedly render this effect multiple times with different uniforms into the same target without target swapping.
	/// This is useful for example for deferred lighting, where each light is rendered with custom blend state on top of the last.
	pub fn with_instances(mut self, instances: Vec<InstanceUniforms>) -> Self {
		self.props.instances = instances;
		self
	}

	pub fn with_blend_state(mut self, blend_state: wgpu::BlendState) -> Self {
		self.props.blend_state = blend_state;
		self
	}
}
