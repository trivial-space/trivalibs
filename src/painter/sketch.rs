use super::{form::Form, shade::Shade, uniform::Uniform, Painter};
use std::collections::HashMap;

pub(crate) struct SketchStorage {
	pub uniforms: HashMap<u32, Uniform>,
	pub instances: Vec<HashMap<u32, Uniform>>,
	pub form: Form,
	pub shade: Shade,
	pub pipeline_key: String,
	pub depth_test: bool,
	pub cull_mode: Option<wgpu::Face>,
	pub blend_state: wgpu::BlendState,
}

pub struct SketchProps {
	pub uniforms: HashMap<u32, Uniform>,
	pub instances: Vec<HashMap<u32, Uniform>>,
	pub cull_mode: Option<wgpu::Face>,
	pub depth_test: bool,
	pub blend_state: wgpu::BlendState,
}

impl Default for SketchProps {
	fn default() -> Self {
		SketchProps {
			uniforms: HashMap::with_capacity(0),
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

		let pipeline_key = format!(
			"s{}-ft{}-ff{}-pd{}-bad{}-bas{}-bao{}-bcd{}-bcs{}-bco{}",
			shade.0,
			f.props.topology as u16,
			f.props.front_face as u16,
			props.depth_test,
			props.blend_state.alpha.dst_factor as u8,
			props.blend_state.alpha.src_factor as u8,
			props.blend_state.alpha.operation as u8,
			props.blend_state.color.dst_factor as u8,
			props.blend_state.color.src_factor as u8,
			props.blend_state.color.operation as u8,
		);

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
