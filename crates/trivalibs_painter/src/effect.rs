use crate::{
	binding::Binding,
	layer::Layer,
	shade::{InstanceData, Shade},
	uniform::Uniform,
	Painter,
};

pub(crate) struct EffectStorage {
	pub shade: Shade,
	pub uniforms: Vec<(u32, Uniform)>,
	pub layer_uniforms: Vec<(u32, Layer)>,
	pub instances: Vec<InstanceData>,
	pub pipeline_key: Vec<u8>,
	pub blend_state: wgpu::BlendState,
	pub uniform_binding_index: u32,
	pub uniform_bindings: Vec<Binding>,
}

#[derive(Clone)]
pub struct EffectProps {
	pub uniforms: Vec<(u32, Uniform)>,
	pub layer_uniforms: Vec<(u32, Layer)>,
	/// Repeatedly render this effect multiple times with different uniforms into the same target without target swapping.
	/// This is useful for example for deferred lighting, where each light is rendered with custom blend state on top of the last.
	pub instances: Vec<InstanceData>,
	pub blend_state: wgpu::BlendState,
}

impl Default for EffectProps {
	fn default() -> Self {
		EffectProps {
			uniforms: Vec::with_capacity(0),
			layer_uniforms: Vec::with_capacity(0),
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
			layer_uniforms: props.layer_uniforms,
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

	pub fn prepare_uniforms(&self, painter: &mut Painter, layer: Layer) {
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
