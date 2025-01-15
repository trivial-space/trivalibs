use crate::uniform::{LayerLayout, UniformLayout};
use wgpu::BindingType;

pub const UNIFORM_BUFFER_VERT: UniformLayout = UniformLayout {
	visibility: wgpu::ShaderStages::VERTEX,
	binding_type: BindingType::Buffer {
		ty: wgpu::BufferBindingType::Uniform,
		has_dynamic_offset: false,
		min_binding_size: None,
	},
};

pub const UNIFORM_BUFFER_FRAG: UniformLayout = UniformLayout {
	visibility: wgpu::ShaderStages::FRAGMENT,
	binding_type: BindingType::Buffer {
		ty: wgpu::BufferBindingType::Uniform,
		has_dynamic_offset: false,
		min_binding_size: None,
	},
};

pub const UNIFORM_BUFFER_BOTH: UniformLayout = UniformLayout {
	visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
	binding_type: BindingType::Buffer {
		ty: wgpu::BufferBindingType::Uniform,
		has_dynamic_offset: false,
		min_binding_size: None,
	},
};

pub const UNIFORM_TEX2D_VERT: UniformLayout = UniformLayout {
	visibility: wgpu::ShaderStages::VERTEX,
	binding_type: wgpu::BindingType::Texture {
		multisampled: false,
		view_dimension: wgpu::TextureViewDimension::D2,
		sample_type: wgpu::TextureSampleType::Float { filterable: true },
	},
};

pub const UNIFORM_TEX2D_FRAG: UniformLayout = UniformLayout {
	visibility: wgpu::ShaderStages::FRAGMENT,
	binding_type: wgpu::BindingType::Texture {
		multisampled: false,
		view_dimension: wgpu::TextureViewDimension::D2,
		sample_type: wgpu::TextureSampleType::Float { filterable: true },
	},
};

pub const UNIFORM_TEX2D_BOTH: UniformLayout = UniformLayout {
	visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
	binding_type: wgpu::BindingType::Texture {
		multisampled: false,
		view_dimension: wgpu::TextureViewDimension::D2,
		sample_type: wgpu::TextureSampleType::Float { filterable: true },
	},
};

pub const UNIFORM_SAMPLER_VERT: UniformLayout = UniformLayout {
	visibility: wgpu::ShaderStages::VERTEX,
	binding_type: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
};

pub const UNIFORM_SAMPLER_FRAG: UniformLayout = UniformLayout {
	visibility: wgpu::ShaderStages::FRAGMENT,
	binding_type: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
};

pub const UNIFORM_SAMPLER_BOTH: UniformLayout = UniformLayout {
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
