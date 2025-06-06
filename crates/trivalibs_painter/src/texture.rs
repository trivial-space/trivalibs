use std::collections::{BTreeMap, BTreeSet};

use crate::{
	bind_group::BindGroup,
	texture_utils::{generate_mipmap_2d, num_mip_levels},
	Painter,
};
use trivalibs_core::utils::default;
use wgpu::TextureViewDescriptor;

#[derive(Clone, Copy)]
pub enum MipMapCount {
	Full,
	Max(u32),
}

/// # Default Texture2DProps
/// - Format: `Rgba8UnormSrgb` (8-bit RGBA color in sRGB color space)
/// - Usage: `TEXTURE_BINDING | COPY_DST` (can be used as texture and receive data)
#[derive(Clone, Copy)]
pub(crate) struct Texture2DProps {
	pub format: wgpu::TextureFormat,
	pub usage: wgpu::TextureUsages,
	pub mips: Option<MipMapCount>,
}

impl Default for Texture2DProps {
	fn default() -> Self {
		Texture2DProps {
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			mips: None,
		}
	}
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Copy, Clone, Debug)]
pub(crate) enum TexViewKey {
	Default,
	WithAllMips,
	AtMipLevel(u32),
}

impl TexViewKey {
	pub fn make_view(&self, t: &wgpu::Texture) -> wgpu::TextureView {
		match self {
			TexViewKey::Default => t.create_view(&default()),
			TexViewKey::WithAllMips => t.create_view(&TextureViewDescriptor {
				mip_level_count: Some(t.mip_level_count()),
				..default()
			}),
			TexViewKey::AtMipLevel(mip_level) => t.create_view(&wgpu::TextureViewDescriptor {
				base_mip_level: *mip_level,
				mip_level_count: Some(1),
				..default()
			}),
		}
	}
}

pub(crate) struct TextureStorage {
	pub texture: wgpu::Texture,
	pub views: BTreeMap<TexViewKey, wgpu::TextureView>,
	pub bindings: BTreeSet<BindGroup>,
}

impl TextureStorage {
	pub(crate) fn prepare_view(&mut self, key: TexViewKey) {
		let view = key.make_view(&self.texture);
		self.views.insert(key, view);
	}
}

#[derive(Clone, Copy)]
pub struct Texture(pub(crate) usize);

fn create_2d(
	painter: &mut Painter,
	width: u32,
	height: u32,
	props: Texture2DProps,
	multi_sampled: bool,
) -> wgpu::Texture {
	let extent = wgpu::Extent3d {
		width,
		height,
		depth_or_array_layers: 1,
	};

	let mip_level_count = if let Some(mips) = props.mips {
		let max_mip_levels = num_mip_levels(extent);
		match mips {
			MipMapCount::Full => max_mip_levels,
			MipMapCount::Max(max) => max.min(max_mip_levels),
		}
	} else {
		1
	};

	painter.device.create_texture(&wgpu::TextureDescriptor {
		label: None,
		size: extent,
		mip_level_count,
		sample_count: if multi_sampled { 4 } else { 1 },
		dimension: wgpu::TextureDimension::D2,
		format: props.format,
		usage: if mip_level_count > 1 {
			props.usage | wgpu::TextureUsages::RENDER_ATTACHMENT
		} else {
			props.usage
		},
		view_formats: &[],
	})
}

fn create_depth(
	painter: &mut Painter,
	width: u32,
	height: u32,
	multi_sampled: bool,
) -> wgpu::Texture {
	painter.device.create_texture(&wgpu::TextureDescriptor {
		label: None,
		size: wgpu::Extent3d {
			width,
			height,
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
	pub(crate) fn create_2d(
		painter: &mut Painter,
		width: u32,
		height: u32,
		props: Texture2DProps,
		multi_sampled: bool,
	) -> Self {
		let texture = create_2d(painter, width, height, props, multi_sampled);

		let mut storage = TextureStorage {
			texture,
			views: BTreeMap::new(),
			bindings: BTreeSet::new(),
		};

		storage.prepare_view(TexViewKey::AtMipLevel(0));
		storage.prepare_view(TexViewKey::WithAllMips);

		painter.textures.push(storage);

		Self(painter.textures.len() - 1)
	}

	pub(crate) fn replace_2d(
		&self,
		painter: &mut Painter,
		width: u32,
		height: u32,
		props: Texture2DProps,
		multi_sampled: bool,
	) {
		let texture = create_2d(painter, width, height, props, multi_sampled);

		let old = &mut painter.textures[self.0];

		let mut storage = TextureStorage {
			texture,
			views: BTreeMap::new(),
			bindings: old.bindings.clone(),
		};

		storage.prepare_view(TexViewKey::AtMipLevel(0));
		storage.prepare_view(TexViewKey::WithAllMips);

		old.texture.destroy();

		painter.textures[self.0] = storage;

		// self.rebuild_bindings(painter);
	}

	pub fn create_depth(
		painter: &mut Painter,
		width: u32,
		height: u32,
		multi_sampled: bool,
	) -> Self {
		let texture = create_depth(painter, width, height, multi_sampled);

		let mut storage = TextureStorage {
			texture,
			views: BTreeMap::new(),
			bindings: BTreeSet::new(),
		};

		storage.prepare_view(TexViewKey::Default);

		painter.textures.push(storage);

		Self(painter.textures.len() - 1)
	}

	pub fn replace_depth(
		&self,
		painter: &mut Painter,
		width: u32,
		height: u32,
		multi_sampled: bool,
	) {
		let texture = create_depth(painter, width, height, multi_sampled);
		let old = &mut painter.textures[self.0];

		let mut storage = TextureStorage {
			texture,
			bindings: old.bindings.clone(),
			views: BTreeMap::new(),
		};

		storage.prepare_view(TexViewKey::Default);

		old.texture.destroy();

		painter.textures[self.0] = storage;
	}

	pub fn fill_2d(&self, painter: &Painter, data: &[u8]) {
		let texture = &painter.textures[self.0].texture;
		let bytes_per_pixel = texture.format().block_copy_size(None).unwrap();

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
				bytes_per_row: Some(size.width * bytes_per_pixel),
				rows_per_image: Some(size.height),
			},
			size,
		);

		self.update_mips(painter);
	}

	pub fn destroy(self, painter: &mut Painter) {
		let t = &mut painter.textures[self.0];
		t.texture.destroy();
	}

	pub(crate) fn prepare_view(&self, painter: &mut Painter, key: TexViewKey) {
		let t = &painter.textures[self.0];
		if !t.views.contains_key(&key) {
			let view = key.make_view(&t.texture);
			painter.textures[self.0].views.insert(key, view);
		}
	}

	pub(crate) fn prepare_mip_level_views(&self, painter: &mut Painter) {
		let t = &painter.textures[self.0].texture;
		for i in 1..t.mip_level_count() {
			self.prepare_view(painter, TexViewKey::AtMipLevel(i));
		}
	}

	pub(crate) fn view<'a>(&self, painter: &'a Painter, key: &TexViewKey) -> &'a wgpu::TextureView {
		painter.textures[self.0].views.get(key).unwrap()
	}

	pub(crate) fn source_view<'a>(&'a self, painter: &'a Painter) -> &'a wgpu::TextureView {
		self.view(painter, &TexViewKey::WithAllMips)
	}

	pub(crate) fn target_view<'a>(&self, painter: &'a Painter) -> &'a wgpu::TextureView {
		self.view(painter, &TexViewKey::AtMipLevel(0))
	}

	pub fn update_mips(&self, painter: &Painter) {
		let t = &painter.textures[self.0].texture;
		if t.mip_level_count() > 1 {
			let texture = &t.clone();
			generate_mipmap_2d(painter, texture);
		}
	}

	pub fn get_mip_level_count(&self, painter: &Painter) -> u32 {
		painter.textures[self.0].texture.mip_level_count()
	}
}
