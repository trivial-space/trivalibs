use super::{
	sketch::Sketch,
	texture::{Texture, Texture2DProps, TextureDepthProps, UniformTex2D},
	uniform::Uniform,
	Painter,
};
use crate::utils::default;
use std::collections::HashMap;

pub(crate) struct LayerStorage {
	pub target_textures: Vec<Texture>,
	pub target_uniforms: Vec<UniformTex2D>,
	pub sketches: Vec<Sketch>,
	pub depth_texture: Option<Texture>,
	// pub depth_uniform: Option<UniformTex2D>,
	pub width: u32,
	pub height: u32,
	pub use_window_size: bool,
	pub clear_color: Option<wgpu::Color>,
	pub binding_visibility: wgpu::ShaderStages,
	pub pipeline_key: String,
	pub format: wgpu::TextureFormat,
	pub multisampled: bool,
}

pub struct LayerProps {
	pub sketches: Vec<Sketch>,
	pub width: u32,
	pub height: u32,
	pub format: Option<wgpu::TextureFormat>,
	pub clear_color: Option<wgpu::Color>,
	pub binding_visibility: wgpu::ShaderStages,
	pub uniforms: HashMap<u32, Uniform>,
	pub multisampled: bool,
}

impl Default for LayerProps {
	fn default() -> Self {
		LayerProps {
			sketches: Vec::with_capacity(0),
			width: 0,
			height: 0,
			format: None,
			uniforms: HashMap::with_capacity(0),
			binding_visibility: wgpu::ShaderStages::FRAGMENT,
			clear_color: None,
			multisampled: false,
		}
	}
}

#[derive(Clone, Copy)]
pub struct Layer(pub(crate) usize);

impl Layer {
	pub fn new(painter: &mut Painter, props: &LayerProps) -> Self {
		let use_window_size = props.width == 0 || props.height == 0;
		let width = if use_window_size {
			painter.config.width
		} else {
			props.width
		};
		let height = if use_window_size {
			painter.config.height
		} else {
			props.height
		};

		let mut target_texture = Vec::with_capacity(1);

		let format = props.format.unwrap_or(painter.config.format);

		target_texture.push(Texture::create_2d(
			painter,
			&Texture2DProps {
				width,
				height,
				format,
				usage: wgpu::TextureUsages::RENDER_ATTACHMENT
					| wgpu::TextureUsages::TEXTURE_BINDING,
			},
		));
		let len = target_texture.len();

		let mut use_depth: bool = false;
		for s in &props.sketches {
			let sketch = &painter.sketches[s.0];
			if sketch.depth_test {
				use_depth = true;
				break;
			}
		}

		let depth_texture =
			use_depth.then(|| Texture::create_depth(painter, &TextureDepthProps { width, height }));

		let pipeline_key = format!("ft{:?}-ms{}", format, props.multisampled as u8);

		let storage = LayerStorage {
			width,
			height,
			target_textures: target_texture,
			target_uniforms: Vec::with_capacity(len),
			sketches: props.sketches.clone(),
			depth_texture,
			use_window_size,
			clear_color: props.clear_color,
			binding_visibility: props.binding_visibility,
			format,
			pipeline_key,
			multisampled: props.multisampled,
		};

		painter.layers.push(storage);
		Layer(painter.layers.len() - 1)
	}

	pub fn get_uniform(&self, painter: &mut Painter) -> UniformTex2D {
		if let Some(uniform) = painter.layers[self.0].target_uniforms.get(0) {
			return *uniform;
		}
		let visibility = painter.layers[self.0].binding_visibility;
		let uniform = painter.uniform_create_tex(
			&UniformTex2D::get_layout(painter, visibility),
			painter.layers[self.0].target_textures[0],
			&painter.sampler_create(&default()),
		);

		painter.layers[self.0].target_uniforms.push(uniform);
		uniform
	}

	pub fn resize(&mut self, painter: &mut Painter, width: u32, height: u32) {
		let use_window_size = width == 0 || height == 0;
		let width = if use_window_size {
			painter.config.width
		} else {
			width
		};
		let height = if use_window_size {
			painter.config.height
		} else {
			height
		};

		let storage = &mut painter.layers[self.0];
		if storage.width == width && storage.height == height {
			return;
		}

		storage.width = width;
		storage.height = height;
		storage.use_window_size = use_window_size;
		storage.target_uniforms.clear();

		let targets = storage.target_textures.clone();
		let depth_texture = storage.depth_texture.clone();

		for texture in targets.iter() {
			let format = painter.textures[texture.0].texture.format();
			texture.replace_2d(
				painter,
				&Texture2DProps {
					width,
					height,
					format,
					usage: wgpu::TextureUsages::RENDER_ATTACHMENT
						| wgpu::TextureUsages::TEXTURE_BINDING,
				},
			);
		}

		if let Some(depth_texture) = depth_texture {
			depth_texture.replace_depth(painter, &TextureDepthProps { width, height });
		}
	}
}
