use std::collections::BTreeMap;

use super::{shade::Shade, uniform::Uniform, Painter};

pub(crate) struct EffectStorage {
	pub uniforms: BTreeMap<u32, Uniform>,
	pub shade: Shade,
	pub pipeline_key: Vec<u8>,
	pub blend_state: wgpu::BlendState,
}

pub struct EffectProps {
	pub uniforms: BTreeMap<u32, Uniform>,
	pub blend_state: wgpu::BlendState,
}

impl Default for EffectProps {
	fn default() -> Self {
		EffectProps {
			uniforms: BTreeMap::new(),
			blend_state: wgpu::BlendState::REPLACE,
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct Effect(pub(crate) usize);

impl Effect {
	pub fn new(painter: &mut Painter, shade: Shade, props: &EffectProps) -> Self {
		let pipeline_key = vec![
			(shade.0 as u16).to_le_bytes().to_vec(),
			(props.blend_state.alpha.dst_factor as u8)
				.to_le_bytes()
				.to_vec(),
			(props.blend_state.alpha.src_factor as u8)
				.to_le_bytes()
				.to_vec(),
			(props.blend_state.alpha.operation as u8)
				.to_le_bytes()
				.to_vec(),
			(props.blend_state.color.dst_factor as u8)
				.to_le_bytes()
				.to_vec(),
			(props.blend_state.color.src_factor as u8)
				.to_le_bytes()
				.to_vec(),
			(props.blend_state.color.operation as u8)
				.to_le_bytes()
				.to_vec(),
		]
		.into_iter()
		.flatten()
		.collect();

		let effect = EffectStorage {
			uniforms: props.uniforms.clone(),
			shade,
			pipeline_key,
			blend_state: props.blend_state,
		};

		painter.effects.push(effect);

		Self(painter.effects.len() - 1)
	}
}
