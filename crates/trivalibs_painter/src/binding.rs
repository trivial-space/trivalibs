use crate::{
	layer::Layer,
	texture::{TexViewKey, Texture},
	uniform::{InstanceUniforms, LayerLayout, Uniform, UniformLayout},
	Painter,
};
use std::collections::btree_map;

#[derive(Clone, Copy)]
pub(crate) struct BindingLayout(pub(crate) usize);

impl BindingLayout {
	pub(crate) fn swapping_effect_layer(painter: &mut Painter, layer_layout: LayerLayout) -> Self {
		let layout = painter
			.device
			.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				entries: &[wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: layer_layout.visibility,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						view_dimension: wgpu::TextureViewDimension::D2,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
					},
					count: None,
				}],
				label: None,
			});

		painter.binding_layouts.push(layout);

		BindingLayout(painter.binding_layouts.len() - 1)
	}

	pub(crate) fn uniforms(painter: &mut Painter, uniforms: &[UniformLayout]) -> Option<Self> {
		if uniforms.is_empty() {
			return None;
		}

		let layout = painter
			.device
			.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				entries: uniforms
					.iter()
					.enumerate()
					.map(|(i, u)| wgpu::BindGroupLayoutEntry {
						binding: i as u32,
						visibility: u.visibility,
						ty: u.binding_type,
						count: None,
					})
					.collect::<Vec<_>>()
					.as_slice(),
				label: None,
			});

		painter.binding_layouts.push(layout);

		Some(BindingLayout(painter.binding_layouts.len() - 1))
	}
}

pub(crate) enum BindingType {
	Uniforms(BindingLayout),
	Layer(Layer),
}

pub(crate) struct BindingStorage {
	pub(crate) typ: BindingType,
	pub(crate) data: Vec<Uniform>,
	pub(crate) binding: wgpu::BindGroup,
}

fn uniform_to_resource<'a>(
	uniform: &'a Uniform,
	painter: &'a Painter,
) -> wgpu::BindingResource<'a> {
	match uniform {
		Uniform::Tex2D(texture) => {
			let view = texture.source_view(painter);
			wgpu::BindingResource::TextureView(&view)
		}
		Uniform::Tex2DAtMipLevel(texture, mip_level) => {
			let view = texture.view(painter, &TexViewKey::AtMipLevel(*mip_level));
			wgpu::BindingResource::TextureView(&view)
		}
		Uniform::Sampler(sampler) => {
			let sampler = &painter.samplers[sampler.0];
			wgpu::BindingResource::Sampler(&sampler)
		}
		Uniform::Buffer(buffer) => {
			let buffer = &painter.buffers[buffer.0];
			buffer.as_entire_binding()
		}
	}
}

#[derive(Clone, Copy)]
pub struct Binding(pub(crate) usize);

impl Binding {
	pub(crate) fn layer(
		painter: &mut Painter,
		layer: Layer,
		layout: BindingLayout,
		tex: Texture,
	) -> Self {
		let uniform = tex.uniform();

		let layout = &painter.binding_layouts[layout.0];

		let entry = wgpu::BindGroupEntry {
			binding: 0,
			resource: uniform_to_resource(&uniform, painter),
		};

		let bind_group = painter
			.device
			.create_bind_group(&wgpu::BindGroupDescriptor {
				label: None,
				layout,
				entries: &[entry],
			});

		let storage = BindingStorage {
			typ: BindingType::Layer(layer),
			data: vec![uniform],
			binding: bind_group,
		};

		painter.bindings.push(storage);

		let b = Binding(painter.bindings.len() - 1);
		b.update_texture_bindings(painter);
		b
	}

	pub(crate) fn uniforms(
		painter: &mut Painter,
		uniforms_length: usize,
		uniform_layout: Option<BindingLayout>,
		shape_uniforms: &Vec<(u32, Uniform)>,
		shape_instances: &Vec<InstanceUniforms>,
		layer_uniforms: &Vec<(u32, Uniform)>,
	) -> Vec<Self> {
		if uniforms_length == 0 || uniform_layout.is_none() {
			return Vec::with_capacity(0);
		}

		let binding_layout = uniform_layout.unwrap();
		let layout = &painter.binding_layouts[binding_layout.0];

		let mut uniforms = btree_map::BTreeMap::new();

		for (i, u) in layer_uniforms.iter() {
			uniforms.insert(*i, *u);
		}

		for (i, u) in shape_uniforms.iter() {
			uniforms.insert(*i, *u);
		}

		if shape_instances.is_empty() {
			let mut entries = uniforms
				.iter()
				.map(|(i, u)| wgpu::BindGroupEntry {
					binding: *i,
					resource: uniform_to_resource(u, painter),
				})
				.collect::<Vec<_>>();
			entries.sort_by(|a, b| a.binding.cmp(&b.binding));

			let bind_group = painter
				.device
				.create_bind_group(&wgpu::BindGroupDescriptor {
					label: None,
					layout,
					entries: &entries,
				});

			let storage = BindingStorage {
				typ: BindingType::Uniforms(binding_layout),
				data: uniforms.into_values().collect(),
				binding: bind_group,
			};

			painter.bindings.push(storage);

			let b = Binding(painter.bindings.len() - 1);

			b.update_texture_bindings(painter);

			vec![b]
		} else {
			let mut bindings = Vec::with_capacity(shape_instances.len());

			for instance in shape_instances {
				for (i, u) in instance.uniforms.iter() {
					uniforms.insert(*i, *u);
				}

				let mut entries = uniforms
					.iter()
					.map(|(i, u)| wgpu::BindGroupEntry {
						binding: *i,
						resource: uniform_to_resource(u, painter),
					})
					.collect::<Vec<_>>();
				entries.sort_by(|a, b| a.binding.cmp(&b.binding));

				let bind_group = painter
					.device
					.create_bind_group(&wgpu::BindGroupDescriptor {
						label: None,
						layout,
						entries: &entries,
					});

				let storage = BindingStorage {
					typ: BindingType::Uniforms(binding_layout),
					data: uniforms.clone().into_values().collect(),
					binding: bind_group,
				};

				bindings.push(Binding(painter.bindings.len()));

				painter.bindings.push(storage);
			}

			for b in &bindings {
				b.update_texture_bindings(painter);
			}

			bindings
		}
	}

	pub(crate) fn rebuild(&self, painter: &mut Painter) {
		let storage = &painter.bindings[self.0];

		match storage.typ {
			BindingType::Layer(layer) => {
				let b = painter.layers[layer.0].binding_layout;
				let layout = &painter.binding_layouts[b.0];
				let entries = storage
					.data
					.iter()
					.enumerate()
					.map(|(i, u)| wgpu::BindGroupEntry {
						binding: i as u32,
						resource: uniform_to_resource(u, painter),
					})
					.collect::<Vec<_>>();

				let bind_group = painter
					.device
					.create_bind_group(&wgpu::BindGroupDescriptor {
						label: None,
						layout,
						entries: &entries,
					});

				painter.bindings[self.0].binding = bind_group;
			}

			BindingType::Uniforms(layout) => {
				let layout = &painter.binding_layouts[layout.0];
				let entries = storage
					.data
					.iter()
					.enumerate()
					.map(|(i, u)| wgpu::BindGroupEntry {
						binding: i as u32,
						resource: uniform_to_resource(u, painter),
					})
					.collect::<Vec<_>>();

				let bind_group = painter
					.device
					.create_bind_group(&wgpu::BindGroupDescriptor {
						label: None,
						layout,
						entries: &entries,
					});

				painter.bindings[self.0].binding = bind_group;
			}
		}
	}

	pub(crate) fn has_mip_level_texture(&self, painter: &Painter) -> bool {
		let storage = &painter.bindings[self.0];
		storage.data.iter().any(|u| match u {
			Uniform::Tex2DAtMipLevel(_, mip_level) => *mip_level > 0,
			_ => false,
		})
	}

	fn update_texture_bindings(&self, painter: &mut Painter) {
		let storage = &painter.bindings[self.0];
		for u in &storage.data {
			if let Uniform::Tex2D(tex) = u {
				painter.textures[tex.0].bindings.push(*self);
			}
		}
	}
}
