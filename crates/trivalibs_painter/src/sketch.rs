use super::{form::Form, shade::Shade, uniform::Uniform, Painter};
use std::collections::BTreeMap;

pub(crate) struct SketchStorage {
	pub uniforms: BTreeMap<u32, Uniform>,
	pub instances: Vec<BTreeMap<u32, Uniform>>,
	pub form: Form,
	pub shade: Shade,
	pub pipeline_key: Vec<u8>,
	pub depth_test: bool,
	pub cull_mode: Option<wgpu::Face>,
	pub blend_state: wgpu::BlendState,
}

pub struct SketchProps {
	pub uniforms: BTreeMap<u32, Uniform>,
	pub instances: Vec<BTreeMap<u32, Uniform>>,
	pub cull_mode: Option<wgpu::Face>,
	pub depth_test: bool,
	pub blend_state: wgpu::BlendState,
}

impl Default for SketchProps {
	fn default() -> Self {
		SketchProps {
			uniforms: BTreeMap::new(),
			instances: Vec::with_capacity(0),
			cull_mode: Some(wgpu::Face::Back),
			depth_test: false,
			blend_state: wgpu::BlendState::REPLACE,
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct Sketch(pub(crate) usize);

impl Sketch {
	pub fn new(painter: &mut Painter, form: Form, shade: Shade, props: &SketchProps) -> Self {
		let f = &painter.forms[form.0];

		let pipeline_key = vec![
			(shade.0 as u16).to_le_bytes().to_vec(),
			(f.props.topology as u8).to_le_bytes().to_vec(),
			(f.props.front_face as u8).to_le_bytes().to_vec(),
			(props.depth_test as u8).to_le_bytes().to_vec(),
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

		let sketch = SketchStorage {
			form,
			shade,
			pipeline_key,
			uniforms: props.uniforms.clone(),
			instances: props.instances.clone(),
			depth_test: props.depth_test,
			cull_mode: props.cull_mode,
			blend_state: props.blend_state,
		};

		painter.sketches.push(sketch);

		Sketch(painter.sketches.len() - 1)
	}
}
