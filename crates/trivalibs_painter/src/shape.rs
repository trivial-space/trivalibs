use crate::{
	form::Form,
	shade::{Shade, ShadeData},
	Painter,
};

#[derive(Clone)]
pub(crate) struct ShapeStorage {
	pub form: Form,
	pub shade: Shade,
	pub data: Option<ShadeData>,
	pub instances: Vec<ShadeData>,
	pub pipeline_key: Vec<u8>,
	pub cull_mode: Option<wgpu::Face>,
	pub blend_state: wgpu::BlendState,
	pub uniform_binding_index: u32,
}

#[derive(Clone)]
pub struct ShapeProps {
	pub data: Option<ShadeData>,
	pub instances: Vec<ShadeData>,
	pub cull_mode: Option<wgpu::Face>,
	pub blend_state: wgpu::BlendState,
}

impl Default for ShapeProps {
	fn default() -> Self {
		ShapeProps {
			data: None,
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
			data: props.data,
			instances: props.instances,
			cull_mode: props.cull_mode,
			blend_state: props.blend_state,
			uniform_binding_index: s.layer_layouts.len() as u32,
		};

		painter.shapes.push(sketch);

		Shape(painter.shapes.len() - 1)
	}
}
