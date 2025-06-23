pub use wgpu;
pub use winit;

pub mod painter;
pub use painter::Painter;
pub mod app;
pub mod app_state;
pub mod bind_group;
pub mod binding;
pub mod binding_constants;
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
pub mod utils;
pub(crate) mod window_dimensions;

pub mod prelude {
	pub use crate::{
		app::{AppConfig, CanvasApp, Event, TempDevStatePersistence},
		app_state,
		binding::{BindingBuffer, InstanceBinding, Mat3U, Vec3U},
		binding_constants::*,
		effect::EffectProps,
		form::FormProps,
		layer::{Layer, LayerProps},
		load_fragment_shader, load_vertex_shader,
		painter::Painter,
		sampler::{Sampler, SamplerProps},
		shade::{Shade, ShadeEffectProps, ShadeProps},
		shape::{Shape, ShapeProps},
		wgpu::{self, SurfaceError, TextureFormat::*, VertexFormat::*},
	};
}
