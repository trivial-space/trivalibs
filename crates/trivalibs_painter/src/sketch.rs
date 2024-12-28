use crate::{form::Form, shade::Shade, uniform::Uniform, Painter};
use std::collections::BTreeMap;

#[derive(Clone)]
pub(crate) struct SketchStorage {
	pub uniforms: BTreeMap<u32, Uniform>,
	pub instances: Vec<BTreeMap<u32, Uniform>>,
	pub form: Form,
	pub shade: Shade,
	pub pipeline_key: Vec<u8>,
	pub cull_mode: Option<wgpu::Face>,
	pub blend_state: wgpu::BlendState,
}

#[derive(Clone)]
pub struct SketchProps {
	pub uniforms: BTreeMap<u32, Uniform>,
	pub instances: Vec<BTreeMap<u32, Uniform>>,
	pub cull_mode: Option<wgpu::Face>,
	pub blend_state: wgpu::BlendState,
}

impl Default for SketchProps {
	fn default() -> Self {
		SketchProps {
			uniforms: BTreeMap::new(),
			instances: Vec::with_capacity(0),
			cull_mode: Some(wgpu::Face::Back),
			blend_state: wgpu::BlendState::REPLACE,
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct Sketch(pub(crate) usize);

impl Sketch {
	pub fn new(painter: &mut Painter, form: Form, shade: Shade, props: SketchProps) -> Self {
		let f = &painter.forms[form.0];

		let pipeline_key = vec![
			(shade.0 as u16).to_le_bytes().to_vec(),
			vec![f.props.topology as u8],
			vec![f.props.front_face as u8],
			vec![props.blend_state.alpha.dst_factor as u8],
			vec![props.blend_state.alpha.src_factor as u8],
			vec![props.blend_state.alpha.operation as u8],
			vec![props.blend_state.color.dst_factor as u8],
			vec![props.blend_state.color.src_factor as u8],
			vec![props.blend_state.color.operation as u8],
		]
		.into_iter()
		.flatten()
		.collect();

		let sketch = SketchStorage {
			form,
			shade,
			pipeline_key,
			uniforms: props.uniforms.clone(),
			instances: props.instances.clone(),
			cull_mode: props.cull_mode,
			blend_state: props.blend_state,
		};

		painter.sketches.push(sketch);

		Sketch(painter.sketches.len() - 1)
	}
}
