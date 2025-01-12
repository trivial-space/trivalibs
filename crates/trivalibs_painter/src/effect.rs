use crate::{shade::Shade, uniform::Uniform, Painter};

pub(crate) struct EffectStorage {
	pub uniforms: Vec<(u32, Uniform)>,
	pub instances: Vec<Vec<(u32, Uniform)>>,
	pub shade: Shade,
	pub pipeline_key: Vec<u8>,
	pub blend_state: wgpu::BlendState,
}

#[derive(Clone)]
pub struct EffectProps {
	pub uniforms: Vec<(u32, Uniform)>,
	/// Repeatedly render this effect multiple times with different uniforms into the same target without target swapping.
	/// This is useful for example for deferred lighting, where each light is rendered with custom blend state on top of the last.
	pub instances: Vec<Vec<(u32, Uniform)>>,
	pub blend_state: wgpu::BlendState,
}

impl Default for EffectProps {
	fn default() -> Self {
		EffectProps {
			uniforms: Vec::with_capacity(0),
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
			uniforms: props.uniforms,
			instances: props.instances,
			shade,
			pipeline_key,
			blend_state: props.blend_state,
		};

		painter.effects.push(effect);

		Self(painter.effects.len() - 1)
	}
}
