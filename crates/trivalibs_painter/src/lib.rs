pub use wgpu;
pub use winit;

pub mod painter;
pub use painter::Painter;
pub mod app;
pub mod binding;
pub mod effect;
pub mod form;
pub mod layer;
pub(crate) mod pipeline;
pub mod sampler;
pub mod shade;
pub mod shaders;
pub mod shape;
pub mod texture;
pub mod texture_utils;
pub mod uniform;
pub mod uniform_constants;
pub mod utils;
pub(crate) mod window_dimensions;

pub mod prelude {
	pub use crate::{
		app::{AppConfig, CanvasApp, Event},
		effect::EffectProps,
		form::FormProps,
		layer::{Layer, LayerProps},
		load_fragment_shader, load_vertex_shader,
		painter::Painter,
		sampler::{Sampler, SamplerProps},
		shade::{Shade, ShadeEffectProps, ShadeProps},
		shape::{Shape, ShapeProps},
		texture::Texture2DProps,
		uniform::{InstanceUniforms, Mat3U, UniformBuffer, Vec3U},
		uniform_constants::*,
		wgpu::{self, SurfaceError, TextureFormat::*, VertexFormat::*},
	};
}
