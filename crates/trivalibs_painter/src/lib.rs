pub use wgpu;
pub use winit;

pub mod painter;
pub use painter::Painter;
pub mod app;
pub mod binding;
pub mod effect;
pub mod form;
pub mod layer;
pub mod shade;
pub mod shaders;
pub mod shape;
pub mod texture;
pub mod uniform;

pub mod prelude {
	pub use crate::{
		app::{AppConfig, CanvasApp, Event},
		effect::EffectProps,
		form::FormProps,
		layer::{Layer, LayerProps},
		load_fragment_shader, load_vertex_shader,
		painter::Painter,
		shade::{InstanceData, Shade, ShadeEffectProps, ShadeProps},
		shape::{Shape, ShapeProps},
		texture::{SamplerProps, Texture2DProps, TextureDepthProps},
		uniform::{Mat3U, UniformBuffer, UniformType, Vec3U},
		wgpu::{self, SurfaceError, TextureFormat::*, VertexFormat::*},
	};
}
