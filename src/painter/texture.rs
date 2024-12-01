use wgpu::BindGroupLayout;

use super::Painter;

pub struct Texture2DProps {
	pub width: u32,
	pub height: u32,
	pub format: wgpu::TextureFormat,
	pub usage: wgpu::TextureUsages,
}

pub struct SamplerProps {
	pub address_mode_u: wgpu::AddressMode,
	pub address_mode_v: wgpu::AddressMode,
	pub mag_filter: wgpu::FilterMode,
	pub min_filter: wgpu::FilterMode,
}

pub(crate) struct TextureStorage {
	pub texture: wgpu::Texture,
}

pub struct UniformTex2D {
	pub texture: Texture,
	pub binding: wgpu::BindGroup,
}

#[derive(Clone, Copy)]
pub struct Texture(pub(crate) usize);

impl Texture {
	pub fn create_2d(painter: &mut Painter, props: &Texture2DProps) -> Self {
		let texture = painter.device.create_texture(&wgpu::TextureDescriptor {
			label: None,
			size: wgpu::Extent3d {
				width: props.width,
				height: props.height,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: props.format,
			usage: props.usage,

			// The base format (Rgba8UnormSrgb) is
			// always supported. Note that using a different
			// texture format is not supported on the WebGL2
			// backend.
			view_formats: &[],
		});

		let storage = TextureStorage { texture };
		painter.textures.push(storage);

		Self(painter.textures.len() - 1)
	}

	pub fn fill_2d(&self, painter: &Painter, data: &[u8]) {
		let texture = &painter.textures[self.0].texture;

		let size = texture.size();
		painter.queue.write_texture(
			// Tells wgpu where to copy the pixel data
			wgpu::ImageCopyTexture {
				texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			// The actual pixel data
			data,
			// The layout of the texture
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4 * size.width),
				rows_per_image: Some(size.height),
			},
			size,
		);
	}

	pub fn get_2d_uniform_layout(
		painter: &Painter,
		visibility: wgpu::ShaderStages,
	) -> wgpu::BindGroupLayout {
		painter
			.device
			.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility,
						ty: wgpu::BindingType::Texture {
							multisampled: false,
							view_dimension: wgpu::TextureViewDimension::D2,
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
						count: None,
					},
				],
				label: None,
			})
	}

	pub fn get_uniform(
		&self,
		painter: &Painter,
		layout: &BindGroupLayout,
		sampler: &wgpu::Sampler,
	) -> UniformTex2D {
		let tex = &painter.textures[self.0].texture;
		let view = tex.create_view(&wgpu::TextureViewDescriptor::default());

		let binding = painter
			.device
			.create_bind_group(&wgpu::BindGroupDescriptor {
				layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&view),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(sampler),
					},
				],
				label: None,
			});

		UniformTex2D {
			texture: *self,
			binding,
		}
	}

	pub fn create_sampler(painter: &Painter, props: &SamplerProps) -> wgpu::Sampler {
		painter.device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: props.address_mode_u,
			address_mode_v: props.address_mode_v,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: props.mag_filter,
			min_filter: props.min_filter,
			mipmap_filter: wgpu::FilterMode::Nearest,
			compare: None,
			..Default::default()
		})
	}
}
