use crate::{
	binding::{Binding, BindingLayout},
	effect::Effect,
	prelude::UNIFORM_LAYER_FRAG,
	shape::Shape,
	texture::{Sampler, Texture, Texture2DProps, TextureDepthProps},
	uniform::{LayerLayout, Uniform},
	Painter,
};

fn map_format_to_u8(format: wgpu::TextureFormat) -> u8 {
	match format {
		wgpu::TextureFormat::R8Unorm => 0,
		wgpu::TextureFormat::R8Snorm => 1,
		wgpu::TextureFormat::R8Uint => 2,
		wgpu::TextureFormat::R8Sint => 3,
		wgpu::TextureFormat::R16Uint => 4,
		wgpu::TextureFormat::R16Sint => 5,
		wgpu::TextureFormat::R16Float => 6,
		wgpu::TextureFormat::Rg8Unorm => 7,
		wgpu::TextureFormat::Rg8Snorm => 8,
		wgpu::TextureFormat::Rg8Uint => 9,
		wgpu::TextureFormat::Rg8Sint => 10,
		wgpu::TextureFormat::R32Uint => 11,
		wgpu::TextureFormat::R32Sint => 12,
		wgpu::TextureFormat::R32Float => 13,
		wgpu::TextureFormat::Rg16Uint => 14,
		wgpu::TextureFormat::Rg16Sint => 15,
		wgpu::TextureFormat::Rg16Float => 16,
		wgpu::TextureFormat::Rgba8Unorm => 17,
		wgpu::TextureFormat::Rgba8UnormSrgb => 18,
		wgpu::TextureFormat::Rgba8Snorm => 19,
		wgpu::TextureFormat::Rgba8Uint => 20,
		wgpu::TextureFormat::Rgba8Sint => 21,
		wgpu::TextureFormat::Bgra8Unorm => 22,
		wgpu::TextureFormat::Bgra8UnormSrgb => 23,
		wgpu::TextureFormat::Rgb10a2Unorm => 24,
		wgpu::TextureFormat::Rg32Uint => 26,
		wgpu::TextureFormat::Rg32Sint => 27,
		wgpu::TextureFormat::Rg32Float => 28,
		wgpu::TextureFormat::Rgba16Uint => 29,
		wgpu::TextureFormat::Rgba16Sint => 30,
		wgpu::TextureFormat::R16Unorm => 31,
		wgpu::TextureFormat::R16Snorm => 32,
		wgpu::TextureFormat::Rg16Unorm => 33,
		wgpu::TextureFormat::Rg16Snorm => 34,
		wgpu::TextureFormat::Rgb9e5Ufloat => 35,
		wgpu::TextureFormat::Rgb10a2Uint => 36,
		wgpu::TextureFormat::Rg11b10Ufloat => 37,
		wgpu::TextureFormat::Rgba16Unorm => 38,
		wgpu::TextureFormat::Rgba16Snorm => 39,
		wgpu::TextureFormat::Rgba16Float => 40,
		wgpu::TextureFormat::Rgba32Uint => 41,
		wgpu::TextureFormat::Rgba32Sint => 42,
		wgpu::TextureFormat::Rgba32Float => 43,
		wgpu::TextureFormat::Stencil8 => 44,
		wgpu::TextureFormat::Depth16Unorm => 45,
		wgpu::TextureFormat::Depth24Plus => 46,
		wgpu::TextureFormat::Depth24PlusStencil8 => 47,
		wgpu::TextureFormat::Depth32Float => 48,
		wgpu::TextureFormat::Depth32FloatStencil8 => 49,
		wgpu::TextureFormat::NV12 => 50,
		wgpu::TextureFormat::Bc1RgbaUnorm => 51,
		wgpu::TextureFormat::Bc1RgbaUnormSrgb => 52,
		wgpu::TextureFormat::Bc2RgbaUnorm => 53,
		wgpu::TextureFormat::Bc2RgbaUnormSrgb => 54,
		wgpu::TextureFormat::Bc3RgbaUnorm => 55,
		wgpu::TextureFormat::Bc3RgbaUnormSrgb => 56,
		wgpu::TextureFormat::Bc4RUnorm => 57,
		wgpu::TextureFormat::Bc4RSnorm => 58,
		wgpu::TextureFormat::Bc5RgUnorm => 59,
		wgpu::TextureFormat::Bc5RgSnorm => 60,
		wgpu::TextureFormat::Bc6hRgbUfloat => 61,
		wgpu::TextureFormat::Bc6hRgbFloat => 62,
		wgpu::TextureFormat::Bc7RgbaUnorm => 63,
		wgpu::TextureFormat::Bc7RgbaUnormSrgb => 64,
		wgpu::TextureFormat::Etc2Rgb8Unorm => 65,
		wgpu::TextureFormat::Etc2Rgb8UnormSrgb => 66,
		wgpu::TextureFormat::Etc2Rgb8A1Unorm => 67,
		wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb => 68,
		wgpu::TextureFormat::Etc2Rgba8Unorm => 69,
		wgpu::TextureFormat::Etc2Rgba8UnormSrgb => 70,
		wgpu::TextureFormat::EacR11Unorm => 71,
		wgpu::TextureFormat::EacR11Snorm => 72,
		wgpu::TextureFormat::EacRg11Unorm => 73,
		wgpu::TextureFormat::EacRg11Snorm => 74,
		wgpu::TextureFormat::Astc {
			block: _,
			channel: _,
		} => 75,
		wgpu::TextureFormat::R64Uint => 76,
	}
}

pub(crate) struct LayerStorage {
	pub shapes: Vec<Shape>,
	pub effects: Vec<Effect>,
	pub binding_layout: BindingLayout,
	pub target_textures: Vec<Texture>,
	pub target_bindings: Vec<Binding>,
	pub depth_texture: Option<Texture>,
	pub width: u32,
	pub height: u32,
	pub use_window_size: bool,
	pub clear_color: Option<wgpu::Color>,
	pub pipeline_key: Vec<u8>,
	pub formats: Vec<wgpu::TextureFormat>,
	pub multisampled_textures: Vec<Texture>,
	pub current_target: usize,
	pub texture_count: usize,
	pub is_multi_target: bool,
	pub uniforms: Vec<(u32, Uniform)>,
	pub layer_uniforms: Vec<(u32, Layer)>,
}

impl LayerStorage {
	pub(crate) fn swap_targets(&mut self) {
		let next = (self.current_target + 1) % self.texture_count;
		self.current_target = next;
	}

	pub(crate) fn current_target<'a>(&'a self) -> &'a Texture {
		&self.target_textures[self.current_target]
	}

	pub(crate) fn current_source<'a>(&'a self) -> &'a Binding {
		let mut idx = self.current_target;
		if idx == 0 {
			idx = self.texture_count;
		}

		&self.target_bindings[idx - 1]
	}
}

#[derive(Clone)]
pub struct LayerProps {
	pub shapes: Vec<Shape>,
	pub effects: Vec<Effect>,
	pub uniforms: Vec<(u32, Uniform)>,
	pub layer_uniforms: Vec<(u32, Layer)>,
	pub width: u32,
	pub height: u32,
	pub sampler: Sampler,
	pub formats: Vec<wgpu::TextureFormat>,
	pub clear_color: Option<wgpu::Color>,
	pub depth_test: bool,
	pub layer_layout: LayerLayout,
	pub multisampled: bool,
}

impl Default for LayerProps {
	fn default() -> Self {
		LayerProps {
			shapes: Vec::with_capacity(0),
			effects: Vec::with_capacity(0),
			uniforms: Vec::with_capacity(0),
			layer_uniforms: Vec::with_capacity(0),
			sampler: Sampler(0),
			width: 0,
			height: 0,
			formats: Vec::with_capacity(1),
			layer_layout: UNIFORM_LAYER_FRAG,
			clear_color: None,
			depth_test: false,
			multisampled: false,
		}
	}
}

#[derive(Clone, Copy)]
pub struct Layer(pub(crate) usize);

impl Layer {
	pub fn new(painter: &mut Painter, props: LayerProps) -> Self {
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

		let depth_texture = props.depth_test.then(|| {
			Texture::create_depth(
				painter,
				TextureDepthProps { width, height },
				props.multisampled,
			)
		});

		let pipeline_key = vec![
			vec![(props.depth_test as u8)],
			vec![props.multisampled as u8],
			props.formats.iter().map(|f| map_format_to_u8(*f)).collect(),
		]
		.into_iter()
		.flatten()
		.collect();

		let layer = Layer(painter.layers.len());

		let use_swap_targets =
			props.effects.len() > 1 || (props.shapes.len() > 0 && props.effects.len() > 0);

		let format_len = props.formats.len();
		let is_multi_target = format_len > 1;

		let texture_count = if is_multi_target {
			format_len
		} else {
			if use_swap_targets {
				2
			} else {
				1
			}
		};

		let mut target_textures = Vec::with_capacity(texture_count);
		let mut target_bindings = Vec::with_capacity(texture_count);
		let mut multisampled_textures =
			Vec::with_capacity(if props.multisampled { texture_count } else { 0 });
		let mut formats = Vec::with_capacity(texture_count);
		let layout = BindingLayout::layer(painter, props.layer_layout);

		if is_multi_target {
			if use_swap_targets {
				panic!("Postprocessing is not supported with multiple targets. Only sketches or one effect can be used.");
			}

			for format in props.formats {
				let tex = Texture::create_2d(
					painter,
					Texture2DProps {
						width,
						height,
						format,
						usage: wgpu::TextureUsages::RENDER_ATTACHMENT
							| wgpu::TextureUsages::TEXTURE_BINDING,
					},
					false,
				);
				target_textures.push(tex);

				target_bindings.push(Binding::layer(painter, layer, layout, tex, props.sampler));

				if props.multisampled {
					multisampled_textures.push(Texture::create_2d(
						painter,
						Texture2DProps {
							width,
							height,
							format,
							usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
						},
						true,
					));
				}

				formats.push(format);
			}
		} else {
			let format = *props.formats.get(0).unwrap_or(&painter.config.format);

			for _ in 0..texture_count {
				let tex = Texture::create_2d(
					painter,
					Texture2DProps {
						width,
						height,
						format,
						usage: wgpu::TextureUsages::RENDER_ATTACHMENT
							| wgpu::TextureUsages::TEXTURE_BINDING,
					},
					false,
				);

				target_textures.push(tex);

				target_bindings.push(Binding::layer(painter, layer, layout, tex, props.sampler));
			}

			if props.multisampled {
				multisampled_textures.push(Texture::create_2d(
					painter,
					Texture2DProps {
						width,
						height,
						format,
						usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
					},
					true,
				));
			}

			formats.push(format);
		}

		let storage = LayerStorage {
			shapes: props.shapes.clone(),
			effects: props.effects.clone(),
			width,
			height,
			target_textures,
			target_bindings,
			depth_texture,
			multisampled_textures,
			use_window_size,
			clear_color: props.clear_color,
			formats,
			pipeline_key,
			current_target: 0,
			texture_count,
			is_multi_target,
			uniforms: props.uniforms,
			layer_uniforms: props.layer_uniforms,
			binding_layout: layout,
		};

		painter.layers.push(storage);

		for s in props.shapes {
			s.prepare_uniforms(painter, layer);
			let key = painter.get_shape_pipeline_key(s, layer);
			painter.ensure_shape_pipeline(&key, s, layer);
		}
		for e in props.effects {
			e.prepare_uniforms(painter, layer);
			let key = painter.get_effect_pipeline_key(e, layer);
			painter.ensure_effect_pipeline(&key, e, layer);
		}

		layer
	}

	pub fn get_target_uniform(&self, painter: &Painter, index: usize) -> Uniform {
		Uniform::Tex2D(painter.layers[self.0].target_textures[index])
	}

	pub fn set_clear_color(&mut self, painter: &mut Painter, color: Option<wgpu::Color>) {
		painter.layers[self.0].clear_color = color;
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

		let targets = storage.target_textures.clone();
		let depth_texture = storage.depth_texture.clone();
		let multisampled_textures = storage.multisampled_textures.clone();

		for texture in targets.iter() {
			let format = painter.textures[texture.0].texture.format();
			texture.replace_2d(
				painter,
				Texture2DProps {
					width,
					height,
					format,
					usage: wgpu::TextureUsages::RENDER_ATTACHMENT
						| wgpu::TextureUsages::TEXTURE_BINDING,
				},
				false,
			);
		}

		if let Some(depth_texture) = depth_texture {
			depth_texture.replace_depth(
				painter,
				TextureDepthProps { width, height },
				!multisampled_textures.is_empty(),
			);
		}

		for t in multisampled_textures {
			let format = painter.textures[t.0].texture.format();
			t.replace_2d(
				painter,
				Texture2DProps {
					width,
					height,
					format,
					usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
				},
				true,
			);
		}
	}
}
