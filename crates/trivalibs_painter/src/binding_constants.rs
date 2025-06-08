use crate::binding::{BindingLayout, LayerLayout};
use wgpu::BindingType;

pub const BINDING_BUFFER_VERT: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::VERTEX,
	binding_type: BindingType::Buffer {
		ty: wgpu::BufferBindingType::Uniform,
		has_dynamic_offset: false,
		min_binding_size: None,
	},
};

pub const BINDING_BUFFER_FRAG: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::FRAGMENT,
	binding_type: BindingType::Buffer {
		ty: wgpu::BufferBindingType::Uniform,
		has_dynamic_offset: false,
		min_binding_size: None,
	},
};

pub const BINDING_BUFFER_BOTH: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
	binding_type: BindingType::Buffer {
		ty: wgpu::BufferBindingType::Uniform,
		has_dynamic_offset: false,
		min_binding_size: None,
	},
};

pub const BINDING_SAMPLER_VERT: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::VERTEX,
	binding_type: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
};

pub const BINDING_SAMPLER_FRAG: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::FRAGMENT,
	binding_type: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
};

pub const BINDING_SAMPLER_BOTH: BindingLayout = BindingLayout {
	visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
	binding_type: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
};

pub const BINDING_LAYER_VERT: LayerLayout = LayerLayout {
	visibility: wgpu::ShaderStages::VERTEX,
};

pub const BINDING_LAYER_FRAG: LayerLayout = LayerLayout {
	visibility: wgpu::ShaderStages::FRAGMENT,
};

pub const BINDING_LAYER_BOTH: LayerLayout = LayerLayout {
	visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
};
