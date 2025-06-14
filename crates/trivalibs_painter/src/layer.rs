use crate::{
	binding::{LayerBinding, LayerLayout, ValueBinding},
	effect::Effect,
	prelude::{BINDING_LAYER_BOTH, BINDING_LAYER_FRAG, BINDING_LAYER_VERT},
	shape::Shape,
	texture::{MipMapCount, Texture, Texture2DProps},
	texture_utils::map_format_to_u8,
	Painter,
};

#[derive(Clone)]
pub struct LayerProps<'a> {
	pub static_texture: bool,
	pub static_texture_data: Option<&'a [u8]>,
	pub shapes: Vec<Shape>,
	pub effects: Vec<Effect>,
	pub bindings: Vec<(u32, ValueBinding)>,
	pub layers: Vec<(u32, LayerBinding)>,
	pub width: u32,
	pub height: u32,
	pub formats: Vec<wgpu::TextureFormat>,
	pub clear_color: Option<wgpu::Color>,
	pub depth_test: bool,
	pub layer_layout: LayerLayout,
	pub multisampled: bool,
	pub mips: Option<MipMapCount>,
}

impl Default for LayerProps<'_> {
	fn default() -> Self {
		LayerProps {
			static_texture: false,
			static_texture_data: None,
			shapes: Vec::with_capacity(0),
			effects: Vec::with_capacity(0),
			bindings: Vec::with_capacity(0),
			layers: Vec::with_capacity(0),
			width: 0,
			height: 0,
			formats: Vec::with_capacity(1),
			layer_layout: BINDING_LAYER_FRAG,
			clear_color: None,
			depth_test: false,
			multisampled: false,
			mips: None,
		}
	}
}

pub(crate) struct LayerStorage {
	pub shapes: Vec<Shape>,
	pub effects: Vec<Effect>,
	pub target_textures: Vec<Texture>,
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
	pub bindings: Vec<(u32, ValueBinding)>,
	pub layers: Vec<(u32, LayerBinding)>,
	pub mips: Option<MipMapCount>,
}

impl LayerStorage {
	pub(crate) fn swap_targets(&mut self) {
		let next = (self.current_target + 1) % self.texture_count;
		self.current_target = next;
	}

	pub(crate) fn current_target_texture<'a>(&'a self) -> &'a Texture {
		&self.target_textures[self.current_target]
	}

	pub(crate) fn current_source_texture<'a>(&'a self) -> &'a Texture {
		let mut idx = self.current_target;
		if idx == 0 {
			idx = self.texture_count;
		}

		&self.target_textures[idx - 1]
	}
}

#[derive(Clone, Copy)]
pub struct Layer(pub(crate) usize);

impl Layer {
	pub fn new(painter: &mut Painter, props: LayerProps) -> Self {
		if props.static_texture && props.shapes.len() > 0 {
			panic!("A layer can only either contain a static texture or render shapes, not both")
		}

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

		let depth_texture = props
			.depth_test
			.then(|| Texture::create_depth(painter, width, height, props.multisampled));

		let pipeline_key = vec![
			vec![(props.depth_test as u8)],
			vec![props.multisampled as u8],
			props.formats.iter().map(|f| map_format_to_u8(*f)).collect(),
		]
		.into_iter()
		.flatten()
		.collect();

		let layer = Layer(painter.layers.len());

		let swapping_effect_count = props
			.effects
			.iter()
			.filter(|e| !e.has_mip_target(painter) && !e.has_mip_source(painter))
			.count();

		let use_swap_targets = swapping_effect_count > 1
			|| ((props.shapes.len() > 0 || props.static_texture) && swapping_effect_count > 0);

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
		let mut multisampled_textures =
			Vec::with_capacity(if props.multisampled { texture_count } else { 0 });
		let mut formats = Vec::with_capacity(texture_count);

		let mut usage =
			wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
		if props.static_texture {
			usage |= wgpu::TextureUsages::COPY_DST;
		}

		if is_multi_target {
			if use_swap_targets {
				panic!("Postprocessing is not supported with multiple targets. Only sketches or one effect can be used.");
			}

			for format in props.formats {
				let tex = Texture::create_2d(
					painter,
					width,
					height,
					Texture2DProps {
						format,
						usage,
						mips: props.mips,
					},
					false,
				);
				target_textures.push(tex);

				if props.multisampled {
					multisampled_textures.push(Texture::create_2d(
						painter,
						width,
						height,
						Texture2DProps {
							format,
							usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
							mips: None,
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
					width,
					height,
					Texture2DProps {
						format,
						usage,
						mips: props.mips,
					},
					false,
				);

				target_textures.push(tex);
			}

			if props.multisampled {
				multisampled_textures.push(Texture::create_2d(
					painter,
					width,
					height,
					Texture2DProps {
						format,
						usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
						mips: None,
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
			depth_texture,
			multisampled_textures,
			use_window_size,
			clear_color: props.clear_color,
			formats,
			pipeline_key,
			current_target: 0,
			texture_count,
			is_multi_target,
			bindings: props.bindings,
			layers: props.layers,
			mips: props.mips,
		};

		painter.layers.push(storage);

		for s in props.shapes {
			s.prepare_bindings(painter, layer);
		}
		for e in props.effects.iter() {
			e.prepare_bindings(painter, layer);
		}
		if props.effects.iter().any(|e| e.has_mip_target(painter)) {
			let textures = painter.layers[layer.0].target_textures.clone();
			for t in textures {
				t.prepare_mip_level_views(painter);
			}
		}

		if let Some(data) = props.static_texture_data {
			layer.update_static_data(painter, data);
		}

		layer
	}

	/// This function is called by after the CanvasApp::init function automatically.
	/// If Layers are created dynamically during App runtime, this
	/// method must to be called manually after all shaders are created.
	pub fn init_layer_gpu_pipelines(&self, painter: &mut Painter) {
		let shapes = (&painter.layers[self.0]).shapes.clone();
		let effects = (&painter.layers[self.0]).effects.clone();

		for s in shapes {
			let key = painter.get_shape_pipeline_key(s, *self);
			painter.ensure_shape_pipeline(&key, s, *self);
		}
		for e in effects {
			let key = painter.get_effect_pipeline_key(e, *self);
			painter.ensure_effect_pipeline(&key, e, *self);
		}
	}

	pub fn update_static_data(&self, painter: &Painter, data: &[u8]) {
		painter.layers[self.0]
			.current_source_texture()
			.fill_2d(painter, data);
	}

	pub fn binding(&self) -> LayerBinding {
		LayerBinding::Source(*self)
	}

	pub fn binding_at_mip_level(&self, mip_level: u32) -> LayerBinding {
		LayerBinding::SourceAtMipLevel(*self, mip_level)
	}

	pub fn depth_binding(&self) -> LayerBinding {
		LayerBinding::Depth(*self)
	}

	pub fn binding_at(&self, index: usize) -> LayerBinding {
		LayerBinding::AtIndex(*self, index)
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
		let mips = storage.mips;

		for texture in targets.iter() {
			let format = painter.textures[texture.0].texture.format();
			texture.replace_2d(
				painter,
				width,
				height,
				Texture2DProps {
					format,
					usage: wgpu::TextureUsages::RENDER_ATTACHMENT
						| wgpu::TextureUsages::TEXTURE_BINDING,
					mips,
				},
				false,
			);
		}

		if let Some(depth_texture) = depth_texture {
			depth_texture.replace_depth(painter, width, height, !multisampled_textures.is_empty());
		}

		for t in multisampled_textures {
			let format = painter.textures[t.0].texture.format();
			t.replace_2d(
				painter,
				width,
				height,
				Texture2DProps {
					format,
					usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
					mips: None,
				},
				true,
			);
		}

		let effects = painter.layers[self.0].effects.clone();
		let prepare_effect_mips = effects.iter().any(|e| e.has_mip_target(painter));

		if prepare_effect_mips {
			let textures = painter.layers[self.0].target_textures.clone();
			for t in textures {
				t.prepare_mip_level_views(painter);
			}
		}
	}

	pub fn get_mip_levels_count(&self, painter: &Painter) -> u32 {
		let storage = &painter.layers[self.0];
		storage.target_textures[0].get_mip_level_count(painter)
	}
}

/// A builder for creating a new [`Layer`].
///
/// # Default Configuration values:
/// - `sampler`: Nearest / ClampToEdge
/// - `layer_layout`: BINDING_LAYER_FRAG
/// - `clear_color`: None
/// - `depth_test`: false
/// - `multisampled`: false
///
/// # Example
/// ```
/// let layer = LayerBuilder::new(painter)
///     .with_size(800, 600)
///     .with_shape(rectangle)
///     .with_clear_color(wgpu::Color::BLACK)
///     .create();
/// ```
///
pub struct LayerBuilder<'a, 'b> {
	props: LayerProps<'b>,
	painter: &'a mut Painter,
}

impl<'a, 'b> LayerBuilder<'a, 'b> {
	pub fn new(painter: &'a mut Painter) -> Self {
		LayerBuilder {
			props: LayerProps::default(),
			painter,
		}
	}

	pub fn create(self) -> Layer {
		Layer::new(self.painter, self.props)
	}

	/// Creates a layer and initializes the its gpu pipelines.
	///
	/// Layers created in the App::init function are automatically initialized.
	/// They can use `create` method to create the layer.
	///
	/// Layers created during runtime must be initialized manually.
	pub fn create_and_init(self) -> Layer {
		let layer = Layer::new(self.painter, self.props);
		layer.init_layer_gpu_pipelines(self.painter);
		layer
	}

	pub fn with_static_texture(mut self) -> Self {
		self.props.static_texture = true;
		self
	}

	pub fn with_static_texture_data(mut self, data: &'b [u8]) -> Self {
		self.props.static_texture = true;
		self.props.static_texture_data = Some(data);
		self
	}

	pub fn with_shapes(mut self, shapes: Vec<Shape>) -> Self {
		self.props.shapes = shapes;
		self
	}

	pub fn with_shape(mut self, shape: Shape) -> Self {
		self.props.shapes.push(shape);
		self
	}

	pub fn with_effects(mut self, effects: Vec<Effect>) -> Self {
		self.props.effects = effects;
		self
	}

	pub fn with_effect(mut self, effect: Effect) -> Self {
		self.props.effects.push(effect);
		self
	}

	pub fn with_bindings(mut self, bindings: Vec<(u32, ValueBinding)>) -> Self {
		self.props.bindings = bindings;
		self
	}

	pub fn with_layers(mut self, layers: Vec<(u32, LayerBinding)>) -> Self {
		self.props.layers = layers;
		self
	}

	pub fn with_size(mut self, width: u32, height: u32) -> Self {
		self.props.width = width;
		self.props.height = height;
		self
	}

	pub fn with_formats(mut self, formats: Vec<wgpu::TextureFormat>) -> Self {
		self.props.formats = formats;
		self
	}

	pub fn with_format(mut self, format: wgpu::TextureFormat) -> Self {
		self.props.formats = vec![format];
		self
	}

	pub fn with_clear_color(mut self, color: wgpu::Color) -> Self {
		self.props.clear_color = Some(color);
		self
	}

	pub fn with_depth_test(mut self) -> Self {
		self.props.depth_test = true;
		self
	}

	pub fn with_layer_layout_vert(mut self) -> Self {
		self.props.layer_layout = BINDING_LAYER_VERT;
		self
	}

	pub fn with_layer_layout_both(mut self) -> Self {
		self.props.layer_layout = BINDING_LAYER_BOTH;
		self
	}

	pub fn with_multisampling(mut self) -> Self {
		self.props.multisampled = true;
		self
	}

	pub fn with_mips(mut self) -> Self {
		self.props.mips = Some(MipMapCount::Full);
		self
	}

	pub fn with_mips_max(mut self, max: u32) -> Self {
		self.props.mips = Some(MipMapCount::Max(max));
		self
	}
}
