use crate::{binding::Binding, uniform::Uniform, Painter};
use trivalibs_core::utils::default;

#[derive(Clone, Copy)]
pub struct Texture2DProps {
	pub width: u32,
	pub height: u32,
	pub format: wgpu::TextureFormat,
	pub usage: wgpu::TextureUsages,
}

#[derive(Clone, Copy)]
pub struct TextureDepthProps {
	pub width: u32,
	pub height: u32,
}

#[derive(Clone, Copy)]
pub struct SamplerProps {
	pub address_mode_u: wgpu::AddressMode,
	pub address_mode_v: wgpu::AddressMode,
	pub mag_filter: wgpu::FilterMode,
	pub min_filter: wgpu::FilterMode,
	pub sample_depth: bool,
}

impl Default for SamplerProps {
	fn default() -> Self {
		Self::NEAREST
	}
}

impl SamplerProps {
	pub const NEAREST: SamplerProps = SamplerProps {
		address_mode_u: wgpu::AddressMode::ClampToEdge,
		address_mode_v: wgpu::AddressMode::ClampToEdge,
		mag_filter: wgpu::FilterMode::Nearest,
		min_filter: wgpu::FilterMode::Nearest,
		sample_depth: false,
	};

	pub const LINEAR: SamplerProps = SamplerProps {
		address_mode_u: wgpu::AddressMode::ClampToEdge,
		address_mode_v: wgpu::AddressMode::ClampToEdge,
		mag_filter: wgpu::FilterMode::Linear,
		min_filter: wgpu::FilterMode::Linear,
		sample_depth: false,
	};
}

pub(crate) struct TextureStorage {
	pub texture: wgpu::Texture,
	pub view: wgpu::TextureView,
	pub bindings: Vec<Binding>,
}

#[derive(Clone, Copy)]
pub struct Texture(pub(crate) usize);

fn create_2d(painter: &mut Painter, props: Texture2DProps, multi_sampled: bool) -> wgpu::Texture {
	painter.device.create_texture(&wgpu::TextureDescriptor {
		label: None,
		size: wgpu::Extent3d {
			width: props.width,
			height: props.height,
			depth_or_array_layers: 1,
		},
		mip_level_count: 1,
		sample_count: if multi_sampled { 4 } else { 1 },
		dimension: wgpu::TextureDimension::D2,
		format: props.format,
		usage: props.usage,
		view_formats: &[],
	})
}

fn create_depth(
	painter: &mut Painter,
	props: TextureDepthProps,
	multi_sampled: bool,
) -> wgpu::Texture {
	painter.device.create_texture(&wgpu::TextureDescriptor {
		label: None,
		size: wgpu::Extent3d {
			width: props.width,
			height: props.height,
			depth_or_array_layers: 1,
		},
		mip_level_count: 1,
		sample_count: if multi_sampled { 4 } else { 1 },
		dimension: wgpu::TextureDimension::D2,
		format: wgpu::TextureFormat::Depth24Plus,
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
		view_formats: &[],
	})
}

impl Texture {
	pub fn create_2d(painter: &mut Painter, props: Texture2DProps, multi_sampled: bool) -> Self {
		let texture = create_2d(painter, props, multi_sampled);
		let view = texture.create_view(&default());
		let storage = TextureStorage {
			texture,
			view,
			bindings: Vec::with_capacity(16),
		};
		painter.textures.push(storage);

		Self(painter.textures.len() - 1)
	}

	pub fn replace_2d(&self, painter: &mut Painter, props: Texture2DProps, multi_sampled: bool) {
		let texture = create_2d(painter, props, multi_sampled);
		let view = texture.create_view(&default());

		let old = &mut painter.textures[self.0];

		let storage = TextureStorage {
			texture,
			view,
			bindings: old.bindings.clone(),
		};

		old.texture.destroy();

		painter.textures[self.0] = storage;

		self.rebuild_bindings(painter);
	}

	pub fn create_depth(
		painter: &mut Painter,
		props: TextureDepthProps,
		multi_sampled: bool,
	) -> Self {
		let texture = create_depth(painter, props, multi_sampled);
		let view = texture.create_view(&default());

		let storage = TextureStorage {
			texture,
			view,
			bindings: Vec::with_capacity(2),
		};

		painter.textures.push(storage);

		Self(painter.textures.len() - 1)
	}

	pub fn replace_depth(
		&self,
		painter: &mut Painter,
		props: TextureDepthProps,
		multi_sampled: bool,
	) {
		let texture = create_depth(painter, props, multi_sampled);
		let view = texture.create_view(&default());
		let old = &mut painter.textures[self.0];

		let storage = TextureStorage {
			texture,
			view,
			bindings: old.bindings.clone(),
		};

		old.texture.destroy();

		painter.textures[self.0] = storage;

		self.rebuild_bindings(painter);
	}

	pub fn fill_2d(&self, painter: &Painter, data: &[u8]) {
		let texture = &painter.textures[self.0].texture;

		let size = texture.size();
		painter.queue.write_texture(
			// Tells wgpu where to copy the pixel data
			wgpu::TexelCopyTextureInfo {
				texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			// The actual pixel data
			data,
			// The layout of the texture
			wgpu::TexelCopyBufferLayout {
				offset: 0,
				bytes_per_row: Some(4 * size.width),
				rows_per_image: Some(size.height),
			},
			size,
		);
	}

	pub fn destroy(self, painter: &mut Painter) {
		let t = &mut painter.textures[self.0];
		t.texture.destroy();
	}

	pub fn uniform(&self) -> Uniform {
		Uniform::Tex2D(*self)
	}

	// Suggestion: Do not recreate bindings multiple time, if they reference several textures.
	// Instead mark them as dirty and rebuild them later.
	pub(crate) fn rebuild_bindings(&self, painter: &mut Painter) {
		let t = &painter.textures[self.0];
		for b in t.bindings.clone() {
			b.rebuild(painter);
		}
	}
}

#[derive(Clone, Copy)]
pub struct Sampler(pub(crate) usize);

impl Sampler {
	pub fn create(painter: &mut Painter, props: SamplerProps) -> Self {
		let sampler = painter.device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: props.address_mode_u,
			address_mode_v: props.address_mode_v,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: props.mag_filter,
			min_filter: props.min_filter,
			mipmap_filter: wgpu::FilterMode::Nearest,
			compare: props.sample_depth.then(|| wgpu::CompareFunction::LessEqual),
			..Default::default()
		});

		painter.samplers.push(sampler);

		Self(painter.samplers.len() - 1)
	}

	pub fn uniform(&self) -> Uniform {
		Uniform::Sampler(*self)
	}
}
