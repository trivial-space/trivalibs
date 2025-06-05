use crate::{
	binding::{BindingLayout, InstanceBinding, LayerBinding, LayerLayout, ValueBinding},
	Painter,
};
use std::collections::btree_map;

#[derive(Clone, Copy)]
pub(crate) struct BindGroupLayout(pub(crate) usize);

impl BindGroupLayout {
	pub(crate) fn layers(painter: &mut Painter, layouts: &[LayerLayout]) -> Option<Self> {
		if layouts.is_empty() {
			return None;
		}

		let layout = painter
			.device
			.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				entries: layouts
					.iter()
					.enumerate()
					.map(|(i, l)| wgpu::BindGroupLayoutEntry {
						binding: i as u32,
						visibility: l.visibility,
						ty: wgpu::BindingType::Texture {
							multisampled: false,
							view_dimension: wgpu::TextureViewDimension::D2,
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
						},
						count: None,
					})
					.collect::<Vec<_>>()
					.as_slice(),
				label: None,
			});

		painter.bind_group_layouts.push(layout);

		Some(BindGroupLayout(painter.bind_group_layouts.len() - 1))
	}

	pub(crate) fn values(painter: &mut Painter, layouts: &[BindingLayout]) -> Option<Self> {
		if layouts.is_empty() {
			return None;
		}

		let layout = painter
			.device
			.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				entries: layouts
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

		painter.bind_group_layouts.push(layout);

		Some(BindGroupLayout(painter.bind_group_layouts.len() - 1))
	}
}

#[derive(Clone)]
pub(crate) struct ValuesBindGroupData {
	pub layout: BindGroupLayout,
	pub data: Vec<Vec<ValueBinding>>,
}

impl ValuesBindGroupData {
	pub(crate) fn from_bindings(
		bindings_length: usize,
		bind_group_layout: Option<BindGroupLayout>,
		shape_bindings: &Vec<(u32, ValueBinding)>,
		shape_instances: &Vec<InstanceBinding>,
		layer_bindings: &Vec<(u32, ValueBinding)>,
	) -> Option<Self> {
		if bindings_length == 0 || bind_group_layout.is_none() {
			return None;
		}

		let layout = bind_group_layout.unwrap();

		let mut binding_map = btree_map::BTreeMap::new();

		for (i, u) in layer_bindings.iter() {
			binding_map.insert(*i, *u);
		}

		for (i, u) in shape_bindings.iter() {
			binding_map.insert(*i, *u);
		}

		if shape_instances.is_empty() {
			let mut bindings = binding_map.iter().collect::<Vec<_>>();
			bindings.sort_by(|(a, _), (b, _)| a.cmp(b));
			let bindings = bindings.iter().map(|(_, b)| **b).collect::<Vec<_>>();

			Some(Self {
				layout,
				data: vec![bindings],
			})
		} else {
			let mut instances = Vec::with_capacity(shape_instances.len());

			for instance in shape_instances {
				for (i, u) in instance.bindings.iter() {
					binding_map.insert(*i, *u);
				}

				let mut bindings = binding_map.iter().collect::<Vec<_>>();
				bindings.sort_by(|(a, _), (b, _)| a.cmp(b));
				let bindings = bindings.iter().map(|(_, b)| **b).collect::<Vec<_>>();

				instances.push(bindings);
			}

			Some(Self {
				layout,
				data: instances,
			})
		}
	}

	pub(crate) fn to_gpu_bind_groups(&self, painter: &Painter) -> Vec<wgpu::BindGroup> {
		self.data
			.iter()
			.map(|u| {
				let entries = u
					.iter()
					.enumerate()
					.map(|(i, u)| wgpu::BindGroupEntry {
						binding: i as u32,
						resource: value_to_resource(u, painter),
					})
					.collect::<Vec<_>>();

				painter
					.device
					.create_bind_group(&wgpu::BindGroupDescriptor {
						label: None,
						layout: &painter.bind_group_layouts[self.layout.0],
						entries: &entries,
					})
			})
			.collect::<Vec<_>>()
	}
}

#[derive(Clone)]
pub(crate) struct LayerBindGroupData {
	pub layout: BindGroupLayout,
	pub data: Vec<LayerBinding>,
}

impl LayerBindGroupData {
	pub(crate) fn from_bindings(
		bindings_length: usize,
		bind_group_layout: Option<BindGroupLayout>,
		shape_bindings: &Vec<(u32, LayerBinding)>,
		layer_bindings: &Vec<(u32, LayerBinding)>,
	) -> Option<Self> {
		if bindings_length == 0 || bind_group_layout.is_none() {
			return None;
		}

		let layout = bind_group_layout.unwrap();

		let mut binding_map = btree_map::BTreeMap::new();

		for (i, u) in layer_bindings.iter() {
			binding_map.insert(*i, *u);
		}

		for (i, u) in shape_bindings.iter() {
			binding_map.insert(*i, *u);
		}

		let mut bindings = binding_map.iter().collect::<Vec<_>>();
		bindings.sort_by(|(a, _), (b, _)| a.cmp(b));
		let bindings = bindings.iter().map(|(_, b)| **b).collect::<Vec<_>>();

		Some(Self {
			layout,
			data: bindings,
		})
	}

	pub(crate) fn to_gpu_bind_group(&self, painter: &Painter) -> wgpu::BindGroup {
		let entries = self
			.data
			.iter()
			.enumerate()
			.map(|(i, u)| wgpu::BindGroupEntry {
				binding: i as u32,
				resource: layer_to_resource(u, painter),
			})
			.collect::<Vec<_>>();

		painter
			.device
			.create_bind_group(&wgpu::BindGroupDescriptor {
				label: None,
				layout: &painter.bind_group_layouts[self.layout.0],
				entries: &entries,
			})
	}
}

pub(crate) enum BindGroupType {
	Values(ValuesBindGroupData),
	Layers(LayerBindGroupData),
}

pub(crate) struct BindGroupStorage {
	pub(crate) typ: BindGroupType,
	pub(crate) bind_group: wgpu::BindGroup,
}

fn value_to_resource<'a>(
	binding: &'a ValueBinding,
	painter: &'a Painter,
) -> wgpu::BindingResource<'a> {
	match binding {
		ValueBinding::Sampler(sampler) => {
			let sampler = &painter.samplers[sampler.0];
			wgpu::BindingResource::Sampler(&sampler)
		}
		ValueBinding::Buffer(buffer) => {
			let buffer = &painter.buffers[buffer.0];
			buffer.as_entire_binding()
		}
	}
}

fn layer_to_resource<'a>(
	binding: &'a LayerBinding,
	_painter: &'a Painter,
) -> wgpu::BindingResource<'a> {
	match binding {
		LayerBinding::Source(_layer) => {
			// let view = layer.source_view(painter);
			// wgpu::BindingResource::TextureView(&view)
			todo!()
		}
		LayerBinding::SourceAtMipLevel(_layer, _mip_level) => {
			// let view = texture.view(painter, &TexViewKey::AtMipLevel(*mip_level));
			// wgpu::BindingResource::TextureView(&view)
			todo!()
		}
		LayerBinding::Depth(_layer) => {
			todo!()
		}
	}
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct BindGroup(pub(crate) usize);

impl BindGroup {
	pub(crate) fn values_bind_groups(
		painter: &mut Painter,
		bindings_length: usize,
		bind_group_layout: Option<BindGroupLayout>,
		shape_bindings: &Vec<(u32, ValueBinding)>,
		shape_instances: &Vec<InstanceBinding>,
		layer_bindings: &Vec<(u32, ValueBinding)>,
	) -> Vec<Self> {
		let data = ValuesBindGroupData::from_bindings(
			bindings_length,
			bind_group_layout,
			shape_bindings,
			shape_instances,
			layer_bindings,
		);

		if let Some(data) = data {
			let bind_groups = data.to_gpu_bind_groups(painter);
			let mut bind_group_indices = Vec::with_capacity(bind_groups.len());

			for bind_group in bind_groups {
				let index = painter.bind_groups.len();
				painter.bind_groups.push(BindGroupStorage {
					typ: BindGroupType::Values(data.clone()),
					bind_group,
				});
				bind_group_indices.push(BindGroup(index));
			}

			bind_group_indices
		} else {
			Vec::new()
		}
	}

	// pub(crate) fn rebuild(&self, painter: &mut Painter) {
	// 	let storage = &painter.bind_groups[self.0];

	// 	match storage.typ {
	// 		BindGroupType::Layers(_) => {
	// 			todo!()
	// 		}
	// BindGroupType::Layer(layer) | BindGroupType::LayerAtMipLevel(layer, _) => {
	// 	let b = painter.layers[layer.0].binding_layout;
	// 	let layout = &painter.bind_group_layouts[b.0];
	// 	let entries = storage
	// 		.data
	// 		.iter()
	// 		.enumerate()
	// 		.map(|(i, u)| wgpu::BindGroupEntry {
	// 			binding: i as u32,
	// 			resource: value_to_resource(u, painter),
	// 		})
	// 		.collect::<Vec<_>>();

	// 	let bind_group = painter
	// 		.device
	// 		.create_bind_group(&wgpu::BindGroupDescriptor {
	// 			label: None,
	// 			layout,
	// 			entries: &entries,
	// 		});

	// 	painter.bind_groups[self.0].bind_group = bind_group;
	// }
	// 		BindGroupType::Values(layout) => {
	// 			let layout = &painter.bind_group_layouts[layout.0];
	// 			let entries = storage
	// 				.data
	// 				.iter()
	// 				.enumerate()
	// 				.map(|(i, u)| wgpu::BindGroupEntry {
	// 					binding: i as u32,
	// 					resource: value_to_resource(u, painter),
	// 				})
	// 				.collect::<Vec<_>>();

	// 			let bind_group = painter
	// 				.device
	// 				.create_bind_group(&wgpu::BindGroupDescriptor {
	// 					label: None,
	// 					layout,
	// 					entries: &entries,
	// 				});

	// 			painter.bind_groups[self.0].bind_group = bind_group;
	// 		}
	// 	}
	// }

	// pub(crate) fn has_mip_level_texture(&self, painter: &Painter) -> bool {
	// 	let storage = &painter.bind_groups[self.0];
	// 	storage.data.iter().any(|u| match u {
	// 		Uniform::Tex2DAtMipLevel(_, mip_level) => *mip_level > 0,
	// 		_ => false,
	// 	})
	// }

	// fn update_texture_bind_groups(&self, painter: &mut Painter) {
	// 	let storage = &painter.bind_groups[self.0];
	// 	for u in &storage.data {
	// 		match u {
	// 			Uniform::Tex2DAtMipLevel(tex, _) => {
	// 				painter.textures[tex.0].bindings.insert(*self);
	// 			}
	// 			Uniform::Tex2D(tex) => {
	// 				painter.textures[tex.0].bindings.insert(*self);
	// 			}
	// 			_ => {}
	// 		}
	// 	}
	// }
}
