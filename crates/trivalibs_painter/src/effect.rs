use trivalibs_core::utils::default;

use crate::{
	shade::{Shade, ShadeData},
	Painter,
};

pub(crate) struct EffectStorage {
	pub shade: Shade,
	pub data: ShadeData,
	pub instances: Vec<ShadeData>,
	pub pipeline_key: Vec<u8>,
	pub blend_state: wgpu::BlendState,
	pub uniform_binding_index: u32,
}

#[derive(Clone)]
pub struct EffectProps {
	pub data: ShadeData,
	/// Repeatedly render this effect multiple times with different uniforms into the same target without target swapping.
	/// This is useful for example for deferred lighting, where each light is rendered with custom blend state on top of the last.
	pub instances: Vec<ShadeData>,
	pub blend_state: wgpu::BlendState,
}

impl Default for EffectProps {
	fn default() -> Self {
		EffectProps {
			data: default(),
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
			data: props.data,
			instances: props.instances,
			shade,
			pipeline_key,
			blend_state: props.blend_state,
			uniform_binding_index: s.layer_layouts.len() as u32,
		};

		painter.effects.push(effect);

		Self(painter.effects.len() - 1)
	}
}
