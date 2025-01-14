use std::collections::btree_map;

use crate::{
	layer::Layer,
	shade::ShadeData,
	texture::{Sampler, Texture},
	uniform::{LayerLayout, Uniform, UniformLayout},
	Painter,
};

#[derive(Clone, Copy)]
pub(crate) struct BindingLayout(pub(crate) usize);

impl BindingLayout {
	pub(crate) fn layer(painter: &mut Painter, layer_layout: LayerLayout) -> Self {
		let layout = painter
			.device
			.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: layer_layout.visibility,
						ty: wgpu::BindingType::Texture {
							multisampled: false,
							view_dimension: wgpu::TextureViewDimension::D2,
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: layer_layout.visibility,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
						count: None,
					},
				],
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
	pub(crate) binding: Option<wgpu::BindGroup>,
}

#[derive(Clone, Copy)]
pub struct Binding(pub(crate) usize);

impl Binding {
	pub(crate) fn layer(
		painter: &mut Painter,
		layer: Layer,
		layout: BindingLayout,
		tex: Texture,
		sampler: Sampler,
	) -> Self {
		let uniforms = vec![Uniform::Tex2D(tex), Uniform::Sampler(sampler)];

		let layout = &painter.binding_layouts[layout.0];

		let bind_group = painter
			.device
			.create_bind_group(&wgpu::BindGroupDescriptor {
				label: None,
				layout,
				entries: &[],
			});

		let storage = BindingStorage {
			typ: BindingType::Layer(layer),
			data: uniforms,
			binding: Some(bind_group),
		};

		painter.bindings.push(storage);

		return Binding(painter.bindings.len() - 1);
	}

	pub(crate) fn uniforms(
		painter: &mut Painter,
		uniforms_length: usize,
		uniform_layout: Option<BindingLayout>,
		shape_data: &Option<ShadeData>,
		shape_instances: &Vec<ShadeData>,
		layer_data: &Option<ShadeData>,
	) -> Vec<Self> {
		if uniforms_length == 0 || uniform_layout.is_none() {
			return Vec::with_capacity(0);
		}

		let binding_layout = uniform_layout.unwrap();
		let layout = &painter.binding_layouts[binding_layout.0];

		let mut uniforms = btree_map::BTreeMap::new();

		if let Some(data) = layer_data {
			for (i, u) in data.uniforms.iter() {
				let resource = match u {
					Uniform::Tex2D(tex) => {
						let tex = &painter.textures[tex.0];
						wgpu::BindingResource::TextureView(&tex.view)
					}
					Uniform::Sampler(sampler) => {
						let sampler = &painter.samplers[sampler.0];
						wgpu::BindingResource::Sampler(&sampler)
					}
					Uniform::Buffer(buffer) => {
						let buffer = &painter.buffers[buffer.0];
						buffer.as_entire_binding()
					}
				};
				uniforms.insert(
					*i,
					wgpu::BindGroupEntry {
						binding: *i,
						resource,
					},
				);
			}
		}

		if let Some(data) = shape_data {
			for (i, u) in data.uniforms.iter() {
				let resource = match u {
					Uniform::Tex2D(tex) => {
						let tex = &painter.textures[tex.0];
						wgpu::BindingResource::TextureView(&tex.view)
					}
					Uniform::Sampler(sampler) => {
						let sampler = &painter.samplers[sampler.0];
						wgpu::BindingResource::Sampler(&sampler)
					}
					Uniform::Buffer(buffer) => {
						let buffer = &painter.buffers[buffer.0];
						buffer.as_entire_binding()
					}
				};
				uniforms.insert(
					*i,
					wgpu::BindGroupEntry {
						binding: *i,
						resource,
					},
				);
			}
		}

		let bind_groups = if shape_instances.is_empty() {
			let mut entries = uniforms.values().map(|e| e.clone()).collect::<Vec<_>>();
			entries.sort_by(|a, b| a.binding.cmp(&b.binding));

			let bind_group = painter
				.device
				.create_bind_group(&wgpu::BindGroupDescriptor {
					label: None,
					layout,
					entries: &entries,
				});

			vec![bind_group]
		} else {
			let mut bindings = Vec::with_capacity(shape_instances.len());

			for instance in shape_instances {
				for (i, u) in instance.uniforms.iter() {
					uniforms.insert(
						*i,
						wgpu::BindGroupEntry {
							binding: *i,
							resource: match u {
								Uniform::Tex2D(tex) => {
									let tex = &painter.textures[tex.0];
									wgpu::BindingResource::TextureView(&tex.view)
								}
								Uniform::Sampler(sampler) => {
									let sampler = &painter.samplers[sampler.0];
									wgpu::BindingResource::Sampler(&sampler)
								}
								Uniform::Buffer(buffer) => {
									let buffer = &painter.buffers[buffer.0];
									buffer.as_entire_binding()
								}
							},
						},
					);
				}

				let mut entries = uniforms.values().map(|e| e.clone()).collect::<Vec<_>>();
				entries.sort_by(|a, b| a.binding.cmp(&b.binding));

				let bind_group = painter
					.device
					.create_bind_group(&wgpu::BindGroupDescriptor {
						label: None,
						layout,
						entries: &entries,
					});

				bindings.push(bind_group);
			}

			bindings
		};

		let mut bindings = Vec::with_capacity(bind_groups.len());

		for bind_group in bind_groups {
			let storage = BindingStorage {
				typ: BindingType::Uniforms(binding_layout),
				data: vec![],
				binding: Some(bind_group),
			};

			bindings.push(Binding(painter.bindings.len()));

			painter.bindings.push(storage);
		}

		bindings
	}

	pub(crate) fn bind_group(&self, painter: &mut Painter) -> &wgpu::BindGroup {
		let binding = &painter.bindings[self.0];

		if binding.binding.is_none() {
			todo!()
		}

		todo!()
	}
}
