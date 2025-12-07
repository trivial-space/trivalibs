use crate::{
	Painter,
	bind_group::{BindGroup, LayerBindGroupData},
	binding::{InstanceBinding, LayerBinding, LayerLayout, ValueBinding},
	effect::Effect,
	prelude::{BINDING_LAYER_BOTH, BINDING_LAYER_FRAG, BINDING_LAYER_VERT},
	shade::Shade,
	shape::Shape,
	texture::{MipMapCount, Texture, Texture2DProps},
	texture_utils::map_format_to_u8,
};

/// Describes the instance rendering strategy to use based on binding configurations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InstanceRenderingStrategy {
	/// No instances - both value and layer bindings ≤ 1
	/// Set both bind groups once and do single draw
	NoInstances,
	/// Only value bindings vary - values > 1, layers ≤ 1
	/// Set layer bindings once, iterate through value bindings
	ValueBindingsVary,
	/// Only layer bindings vary - layers > 1, values ≤ 1
	/// Set value bindings once, iterate through layer bindings
	LayerBindingsVary,
	/// Both bindings vary - both > 1
	/// Iterate through all instances, setting both bind groups per draw
	BothBindingsVary,
}

impl InstanceRenderingStrategy {
	/// Determines the rendering strategy based on bind group counts.
	pub(crate) fn from_counts(value_count: usize, layer_count: usize) -> Self {
		match (value_count <= 1, layer_count <= 1) {
			(true, true) => Self::NoInstances,
			(false, true) => Self::ValueBindingsVary,
			(true, false) => Self::LayerBindingsVary,
			(false, false) => Self::BothBindingsVary,
		}
	}
}

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

/// Encapsulates a single shape and its per-layer binding state.
#[derive(Clone)]
pub(crate) struct ShapeData {
	pub shape: Shape,
	pub bind_groups: Vec<BindGroup>,
	pub layer_bind_group_data: Option<LayerBindGroupData>,
	pub rendering_strategy: InstanceRenderingStrategy,
}

impl ShapeData {
	/// Creates a new ShapeData with bindings prepared using explicit layer data.
	pub(crate) fn new(
		painter: &mut Painter,
		shape: Shape,
		layer_bindings: &[(u32, ValueBinding)],
		layer_layers: &[(u32, LayerBinding)],
	) -> Self {
		let mut shape_data = ShapeData {
			shape,
			bind_groups: Vec::new(),
			layer_bind_group_data: None,
			rendering_strategy: InstanceRenderingStrategy::NoInstances,
		};

		shape_data.prepare_value_bindings(painter, layer_bindings);
		shape_data.prepare_layer_bindings(painter, layer_layers);
		shape_data.update_rendering_strategy();

		shape_data
	}

	/// Updates the rendering strategy based on current bind group counts.
	fn update_rendering_strategy(&mut self) {
		let value_count = self.bind_groups.len();
		let layer_count = self
			.layer_bind_group_data
			.as_ref()
			.map(|d| d.data.len())
			.unwrap_or(0);
		self.rendering_strategy = InstanceRenderingStrategy::from_counts(value_count, layer_count);
	}

	/// Prepares value bindings (expensive - creates GPU resources).
	pub(crate) fn prepare_value_bindings(
		&mut self,
		painter: &mut Painter,
		layer_bindings: &[(u32, ValueBinding)],
	) {
		// Extract necessary data from storage
		let sp = &painter.shapes[self.shape.0];
		let shade_idx = sp.shade.0;
		let value_bindings = sp.bindings.clone();
		let instances = sp.instances.clone();

		let sd = &painter.shades[shade_idx];
		let value_bindings_length = sd.value_bindings_length;
		let binding_layout = sd.binding_layout;

		// Create GPU bind groups (expensive operation)
		self.bind_groups = BindGroup::values_bind_groups(
			painter,
			value_bindings_length,
			binding_layout,
			&value_bindings,
			&instances,
			layer_bindings,
		);
	}

	/// Prepares layer bindings (cheap - only descriptors).
	pub(crate) fn prepare_layer_bindings(
		&mut self,
		painter: &Painter,
		layer_layers: &[(u32, LayerBinding)],
	) {
		// Extract necessary data from storage
		let sp = &painter.shapes[self.shape.0];
		let shade_idx = sp.shade.0;
		let shape_layers = sp.layers.as_slice();
		let shape_instances = sp.instances.as_slice();

		let sd = &painter.shades[shade_idx];
		let layer_bindings_length = sd.layer_bindings_length;
		let layers_layout = sd.layers_layout;

		// Create layer bind group data (cheap operation)
		self.layer_bind_group_data = LayerBindGroupData::from_bindings(
			layer_bindings_length,
			layers_layout,
			shape_layers,
			shape_instances,
			layer_layers,
		);
	}
}

/// Encapsulates a single effect and its per-layer binding state.
#[derive(Clone)]
pub(crate) struct EffectData {
	pub effect: Effect,
	pub bind_groups: Vec<BindGroup>,
	pub layer_bind_group_data: Option<LayerBindGroupData>,
	pub rendering_strategy: InstanceRenderingStrategy,
}

impl EffectData {
	/// Creates a new EffectData with bindings prepared using explicit layer data.
	pub(crate) fn new(
		painter: &mut Painter,
		effect: Effect,
		layer_bindings: &[(u32, ValueBinding)],
		layer_layers: &[(u32, LayerBinding)],
	) -> Self {
		let mut effect_data = EffectData {
			effect,
			bind_groups: Vec::new(),
			layer_bind_group_data: None,
			rendering_strategy: InstanceRenderingStrategy::NoInstances,
		};

		effect_data.prepare_value_bindings(painter, layer_bindings);
		effect_data.prepare_layer_bindings(painter, layer_layers);
		effect_data.update_rendering_strategy();

		effect_data
	}

	/// Updates the rendering strategy based on current bind group counts.
	fn update_rendering_strategy(&mut self) {
		let value_count = self.bind_groups.len();
		let layer_count = self
			.layer_bind_group_data
			.as_ref()
			.map(|d| d.data.len())
			.unwrap_or(0);
		self.rendering_strategy = InstanceRenderingStrategy::from_counts(value_count, layer_count);
	}

	/// Prepares value bindings (expensive - creates GPU resources).
	pub(crate) fn prepare_value_bindings(
		&mut self,
		painter: &mut Painter,
		layer_bindings: &[(u32, ValueBinding)],
	) {
		// Extract necessary data from storage
		let ep = &painter.effects[self.effect.0];
		let shade_idx = ep.shade.0;
		let value_bindings = ep.bindings.clone();
		let instances = ep.instances.clone();

		let sd = &painter.shades[shade_idx];
		let value_bindings_length = sd.value_bindings_length;
		let binding_layout = sd.binding_layout;

		// Convert slice to Vec for API compatibility
		let layer_bindings_vec = layer_bindings.to_vec();

		// Create GPU bind groups (expensive operation)
		self.bind_groups = BindGroup::values_bind_groups(
			painter,
			value_bindings_length,
			binding_layout,
			&value_bindings,
			&instances,
			&layer_bindings_vec,
		);
	}

	/// Prepares layer bindings (cheap - only descriptors).
	pub(crate) fn prepare_layer_bindings(
		&mut self,
		painter: &Painter,
		layer_layers: &[(u32, LayerBinding)],
	) {
		// Extract necessary data from storage
		let ep = &painter.effects[self.effect.0];
		let shade_idx = ep.shade.0;
		let effect_layers = ep.layers.clone();
		let effect_instances = ep.instances.as_slice();

		let sd = &painter.shades[shade_idx];
		let layer_bindings_length = sd.layer_bindings_length;
		let layers_layout = sd.layers_layout;

		// Convert slice to Vec for API compatibility
		let layer_layers_vec = layer_layers.to_vec();

		// Create layer bind group data (cheap operation)
		self.layer_bind_group_data = LayerBindGroupData::from_bindings(
			layer_bindings_length,
			layers_layout,
			&effect_layers,
			effect_instances,
			&layer_layers_vec,
		);
	}
}

pub(crate) struct LayerStorage {
	pub shapes: Vec<ShapeData>,
	pub effects: Vec<EffectData>,
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
			if use_swap_targets { 2 } else { 1 }
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
				panic!(
					"Postprocessing is not supported with multiple targets. Only sketches or one effect can be used."
				);
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

		let shape_data: Vec<ShapeData> = props
			.shapes
			.into_iter()
			.map(|shape| ShapeData::new(painter, shape, &props.bindings, &props.layers))
			.collect();

		let effect_data: Vec<EffectData> = props
			.effects
			.iter()
			.map(|effect| EffectData::new(painter, *effect, &props.bindings, &props.layers))
			.collect();

		let storage = LayerStorage {
			shapes: shape_data,
			effects: effect_data,
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
	///
	/// If Layers are created dynamically during App runtime,
	/// or if they need to be rendered directly inside CanvasApp::init,
	/// this method must to be called manually after all shaders are loaded.
	///
	/// Alternatively, LayerBuilder::create_and_init can be used to create and initialize.
	/// Or Painter::init_and_paint can be used to initialize and paint in one call.
	pub fn init_gpu_pipelines(&self, painter: &mut Painter) {
		let shapes: Vec<Shape> = painter.layers[self.0]
			.shapes
			.iter()
			.map(|sd| sd.shape)
			.collect();

		let effects: Vec<Effect> = painter.layers[self.0]
			.effects
			.iter()
			.map(|ed| ed.effect)
			.collect();

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

	/// Updates all layer-level bindings at once.
	///
	/// Layer-level bindings serve as defaults for all shapes and effects in this layer.
	/// These bindings are merged with shape/effect-specific bindings during rendering,
	/// with shape/effect bindings taking precedence.
	///
	/// This method automatically re-prepares all shapes and effects in the layer
	/// to reflect the new bindings.
	///
	/// # Arguments
	/// * `painter` - The painter instance
	/// * `layers` - Vector of (slot_index, LayerBinding) pairs
	///
	pub fn set_layer_bindings(&self, painter: &mut Painter, layers: Vec<(u32, LayerBinding)>) {
		painter.layers[self.0].layers = layers.clone();

		// Clone the shapes and effects to avoid borrow conflicts
		let mut shapes = painter.layers[self.0].shapes.clone();
		let mut effects = painter.layers[self.0].effects.clone();

		// Update ONLY layer bindings (cheap operation - no GPU resource regeneration)
		for sd in &mut shapes {
			sd.prepare_layer_bindings(painter, &layers);
		}

		// Update ONLY layer bindings for effects
		for ed in &mut effects {
			ed.prepare_layer_bindings(painter, &layers);
		}

		// Write back
		painter.layers[self.0].shapes = shapes;
		painter.layers[self.0].effects = effects;
	}

	/// Updates a single layer-level binding by slot index.
	///
	/// This is a convenience method for updating just one binding without
	/// replacing the entire bindings vector. If the slot doesn't exist,
	/// it will be added. If it exists, it will be updated.
	///
	/// # Arguments
	/// * `painter` - The painter instance
	/// * `slot` - The binding slot index
	/// * `binding` - The new LayerBinding for this slot
	///
	pub fn set_layer_binding(&self, painter: &mut Painter, slot: u32, binding: LayerBinding) {
		let mut layers = painter.layers[self.0].layers.clone();

		// Find and update or insert
		if let Some(pos) = layers.iter().position(|(i, _)| *i == slot) {
			layers[pos].1 = binding;
		} else {
			layers.push((slot, binding));
			layers.sort_by_key(|(s, _)| *s);
		}

		self.set_layer_bindings(painter, layers);
	}

	/// Replaces all shapes in the layer with a new list of shapes.
	///
	/// This is useful for dynamically changing which shapes are rendered in a layer at runtime.
	/// The method will prepare bindings for the new shapes and ensure their GPU pipelines exist.
	/// Pipelines are cached and reused, so adding shapes that share configurations with existing
	/// shapes is efficient.
	///
	pub fn set_shapes(&self, painter: &mut Painter, shapes: Vec<Shape>) {
		let layer_bindings = painter.layers[self.0].bindings.clone();
		let layer_layers = painter.layers[self.0].layers.clone();

		painter.layers[self.0].shapes = shapes
			.iter()
			.map(|&shape| ShapeData::new(painter, shape, &layer_bindings, &layer_layers))
			.collect();

		// Ensure pipelines exist for all shapes (will reuse cached if available)
		for shape in shapes {
			let key = painter.get_shape_pipeline_key(shape, *self);
			painter.ensure_shape_pipeline(&key, shape, *self);
		}
	}

	/// Adds a single shape to the layer.
	///
	/// This is a convenience method for appending a shape to the existing shapes vector.
	/// If you need to replace all shapes or add multiple shapes at once, use `set_shapes()` instead.
	///
	pub fn add_shape(&self, painter: &mut Painter, shape: Shape) {
		let layer_bindings = painter.layers[self.0].bindings.clone();
		let layer_layers = painter.layers[self.0].layers.clone();

		let shape_data = ShapeData::new(painter, shape, &layer_bindings, &layer_layers);
		painter.layers[self.0].shapes.push(shape_data);

		// Ensure pipeline exists for this shape
		let key = painter.get_shape_pipeline_key(shape, *self);
		painter.ensure_shape_pipeline(&key, shape, *self);
	}

	/// Removes a specific shape from the layer.
	///
	/// This filters out all occurrences of the given shape from the layer's shape list.
	/// If the shape appears multiple times, all instances will be removed.
	///
	pub fn remove_shape(&self, painter: &mut Painter, shape: Shape) {
		painter.layers[self.0]
			.shapes
			.retain(|sd| sd.shape.0 != shape.0);
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

		let prepare_effect_mips = painter.layers[self.0]
			.effects
			.iter()
			.any(|ed| ed.effect.has_mip_target(painter));

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
	/// Alternatively, use Painter::init_and_paint method to initialize and paint in one call.
	pub fn create_and_init(self) -> Layer {
		let layer = Layer::new(self.painter, self.props);
		layer.init_gpu_pipelines(self.painter);
		layer
	}

	/// Creates a layer, initializes the its gpu pipelines and immediately renders it.
	pub fn create_and_paint(self) -> Layer {
		let layer = Layer::new(self.painter, self.props);
		layer.init_gpu_pipelines(self.painter);
		self.painter.paint(layer);
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

/// A builder for creating a new [`Layer`] with a single [`Effect`].
///
/// This builder merges the functionality of [`LayerBuilder`] and [`EffectBuilder`]
/// for the common case of creating a layer with exactly one effect.
///
/// # Default Configuration values:
/// - `blend_state`: wgpu::BlendState::REPLACE
/// - `clear_color`: None
///
/// # Example
/// ```
/// let layer = SingleEffectBuilder::new(painter, shade)
///     .with_size(800, 600)
///     .with_bindings(vec![(0, some_binding)])
///     .with_clear_color(wgpu::Color::BLACK)
///     .create();
/// ```
///
pub struct SingleEffectLayerBuilder<'a> {
	painter: &'a mut Painter,
	shade: Shade,

	// Effect properties
	effect_bindings: Vec<(u32, ValueBinding)>,
	effect_layers: Vec<(u32, LayerBinding)>,
	effect_instances: Vec<InstanceBinding>,
	blend_state: wgpu::BlendState,
	dst_mip_level: Option<u32>,
	src_mip_level: Option<u32>,

	// Layer properties
	width: u32,
	height: u32,
	format: Option<wgpu::TextureFormat>,
	clear_color: Option<wgpu::Color>,
	mips: Option<MipMapCount>,
}

impl<'a> SingleEffectLayerBuilder<'a> {
	pub fn new(painter: &'a mut Painter, shade: Shade) -> Self {
		SingleEffectLayerBuilder {
			painter,
			shade,
			effect_bindings: Vec::with_capacity(0),
			effect_layers: Vec::with_capacity(0),
			effect_instances: Vec::with_capacity(0),
			blend_state: wgpu::BlendState::REPLACE,
			dst_mip_level: None,
			src_mip_level: None,
			width: 0,
			height: 0,
			format: None,
			clear_color: None,
			mips: None,
		}
	}

	pub fn create(self) -> Layer {
		let (painter, props) = self.into_layer_props();
		Layer::new(painter, props)
	}

	/// Creates a layer and immediately initializes its GPU pipelines.
	///
	/// Layers created during app initialization are initialized automatically,
	/// so they can use `create`. Runtime-created layers must either call
	/// `Layer::init_gpu_pipelines` manually or use this helper instead.
	pub fn create_and_init(self) -> Layer {
		let (painter, props) = self.into_layer_props();
		let layer = Layer::new(painter, props);
		layer.init_gpu_pipelines(painter);
		layer
	}

	// Effect builder methods

	pub fn with_bindings(mut self, bindings: Vec<(u32, ValueBinding)>) -> Self {
		self.effect_bindings = bindings;
		self
	}

	pub fn with_layers(mut self, layers: Vec<(u32, LayerBinding)>) -> Self {
		self.effect_layers = layers;
		self
	}

	/// Repeatedly render this effect multiple times with different bindings into the same target without target swapping.
	/// This is useful for example for deferred lighting, where each light is rendered with custom blend state on top of the last.
	pub fn with_instances(mut self, instances: Vec<InstanceBinding>) -> Self {
		self.effect_instances = instances;
		self
	}

	pub fn with_blend_state(mut self, blend_state: wgpu::BlendState) -> Self {
		self.blend_state = blend_state;
		self
	}

	pub fn with_mip_target(mut self, dst_mip_level: u32) -> Self {
		self.dst_mip_level = Some(dst_mip_level);
		self
	}

	pub fn with_mip_source(mut self, src_mip_level: u32) -> Self {
		self.src_mip_level = Some(src_mip_level);
		self
	}

	// Layer builder methods

	pub fn with_size(mut self, width: u32, height: u32) -> Self {
		self.width = width;
		self.height = height;
		self
	}

	pub fn with_format(mut self, format: wgpu::TextureFormat) -> Self {
		self.format = Some(format);
		self
	}

	pub fn with_clear_color(mut self, color: wgpu::Color) -> Self {
		self.clear_color = Some(color);
		self
	}

	pub fn with_mips(mut self) -> Self {
		self.mips = Some(MipMapCount::Full);
		self
	}

	pub fn with_mips_max(mut self, max: u32) -> Self {
		self.mips = Some(MipMapCount::Max(max));
		self
	}

	fn into_layer_props(self) -> (&'a mut Painter, LayerProps<'a>) {
		let SingleEffectLayerBuilder {
			painter,
			shade,
			effect_bindings,
			effect_layers,
			effect_instances,
			blend_state,
			dst_mip_level,
			src_mip_level,
			width,
			height,
			format,
			clear_color,
			mips,
		} = self;

		let effect = Effect::new(
			painter,
			shade,
			crate::effect::EffectProps {
				bindings: Vec::with_capacity(0),
				layers: Vec::with_capacity(0),
				instances: effect_instances,
				blend_state,
				dst_mip_level,
				src_mip_level,
			},
		);

		let mut formats = Vec::with_capacity(1);
		if let Some(format) = format {
			formats.push(format);
		}

		(
			painter,
			LayerProps {
				static_texture: false,
				static_texture_data: None,
				shapes: Vec::with_capacity(0),
				effects: vec![effect],
				bindings: effect_bindings,
				layers: effect_layers,
				width,
				height,
				formats,
				clear_color,
				depth_test: false,
				layer_layout: BINDING_LAYER_FRAG,
				multisampled: false,
				mips,
			},
		)
	}
}
