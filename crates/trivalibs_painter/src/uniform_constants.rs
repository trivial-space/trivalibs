use crate::binding::{BindingLayout, LayerLayout};
use wgpu::BindingType;

pub const UNIFORM_BUFFER_VERT: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::VERTEX,
	binding_type: BindingType::Buffer {
		ty: wgpu::BufferBindingType::Uniform,
		has_dynamic_offset: false,
		min_binding_size: None,
	},
};

pub const UNIFORM_BUFFER_FRAG: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::FRAGMENT,
	binding_type: BindingType::Buffer {
		ty: wgpu::BufferBindingType::Uniform,
		has_dynamic_offset: false,
		min_binding_size: None,
	},
};

pub const UNIFORM_BUFFER_BOTH: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
	binding_type: BindingType::Buffer {
		ty: wgpu::BufferBindingType::Uniform,
		has_dynamic_offset: false,
		min_binding_size: None,
	},
};

pub const UNIFORM_TEX2D_VERT: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::VERTEX,
	binding_type: wgpu::BindingType::Texture {
		multisampled: false,
		view_dimension: wgpu::TextureViewDimension::D2,
		sample_type: wgpu::TextureSampleType::Float { filterable: true },
	},
};

pub const UNIFORM_TEX2D_FRAG: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::FRAGMENT,
	binding_type: wgpu::BindingType::Texture {
		multisampled: false,
		view_dimension: wgpu::TextureViewDimension::D2,
		sample_type: wgpu::TextureSampleType::Float { filterable: true },
	},
};

pub const UNIFORM_TEX2D_BOTH: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
	binding_type: wgpu::BindingType::Texture {
		multisampled: false,
		view_dimension: wgpu::TextureViewDimension::D2,
		sample_type: wgpu::TextureSampleType::Float { filterable: true },
	},
};

pub const UNIFORM_TEX2D_VERT_NO_FILTER: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::VERTEX,
	binding_type: wgpu::BindingType::Texture {
		multisampled: false,
		view_dimension: wgpu::TextureViewDimension::D2,
		sample_type: wgpu::TextureSampleType::Float { filterable: false },
	},
};

pub const UNIFORM_TEX2D_FRAG_NO_FILTER: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::FRAGMENT,
	binding_type: wgpu::BindingType::Texture {
		multisampled: false,
		view_dimension: wgpu::TextureViewDimension::D2,
		sample_type: wgpu::TextureSampleType::Float { filterable: false },
	},
};

pub const UNIFORM_TEX2D_BOTH_NO_FILTER: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
	binding_type: wgpu::BindingType::Texture {
		multisampled: false,
		view_dimension: wgpu::TextureViewDimension::D2,
		sample_type: wgpu::TextureSampleType::Float { filterable: false },
	},
};

pub const UNIFORM_SAMPLER_VERT: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::VERTEX,
	binding_type: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
};

pub const UNIFORM_SAMPLER_FRAG: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::FRAGMENT,
	binding_type: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
};

pub const UNIFORM_SAMPLER_BOTH: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
	binding_type: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
};

pub const UNIFORM_LAYER_VERT: LayerLayout = LayerLayout {
	visibility: wgpu::ShaderStages::VERTEX,
};

pub const UNIFORM_LAYER_FRAG: LayerLayout = LayerLayout {
	visibility: wgpu::ShaderStages::FRAGMENT,
};

pub const UNIFORM_LAYER_BOTH: LayerLayout = LayerLayout {
	visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
};
