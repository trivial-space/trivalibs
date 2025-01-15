use crate::{
	binding::Binding,
	form::Form,
	layer::Layer,
	shade::{InstanceData, Shade},
	uniform::Uniform,
	Painter,
};

#[derive(Clone)]
pub(crate) struct ShapeStorage {
	pub form: Form,
	pub shade: Shade,
	pub uniforms: Vec<(u32, Uniform)>,
	pub layer_uniforms: Vec<(u32, Layer)>,
	pub instances: Vec<InstanceData>,
	pub pipeline_key: Vec<u8>,
	pub cull_mode: Option<wgpu::Face>,
	pub blend_state: wgpu::BlendState,
	pub uniform_binding_index: u32,
	pub uniform_bindings: Vec<Binding>,
}

#[derive(Clone)]
pub struct ShapeProps {
	pub uniforms: Vec<(u32, Uniform)>,
	pub layer_uniforms: Vec<(u32, Layer)>,
	pub instances: Vec<InstanceData>,
	pub cull_mode: Option<wgpu::Face>,
	pub blend_state: wgpu::BlendState,
}

impl Default for ShapeProps {
	fn default() -> Self {
		ShapeProps {
			uniforms: Vec::with_capacity(0),
			layer_uniforms: Vec::with_capacity(0),
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
		let s = &painter.shades[shade.0];

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
			],
		]
		.into_iter()
		.flatten()
		.collect();

		let sketch = ShapeStorage {
			form,
			shade,
			pipeline_key,
			uniforms: props.uniforms,
			layer_uniforms: props.layer_uniforms,
			instances: props.instances,
			cull_mode: props.cull_mode,
			blend_state: props.blend_state,
			uniform_binding_index: s.layer_layouts.len() as u32,
			uniform_bindings: Vec::with_capacity(0),
		};

		painter.shapes.push(sketch);

		Shape(painter.shapes.len() - 1)
	}

	pub(crate) fn prepare_uniforms(&self, painter: &mut Painter, layer: Layer) {
		let sp = &painter.shapes[self.0];
		let sd = &painter.shades[sp.shade.0];
		let data = &sp.uniforms.clone();
		let instances = &sp.instances.clone();
		let layer_data = &painter.layers[layer.0].uniforms.clone();

		let uniforms = Binding::uniforms(
			painter,
			sd.uniforms_length,
			sd.uniform_layout,
			data,
			instances,
			layer_data,
		);

		painter.shapes[self.0].uniform_bindings = uniforms;
	}
}
