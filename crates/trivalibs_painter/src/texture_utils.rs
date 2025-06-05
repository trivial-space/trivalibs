use crate::{painter::FULL_SCREEN_TEXTURE_PIPELINE, Painter};
use trivalibs_core::utils::default;
use wgpu::StoreOp;

pub fn num_mip_levels(size: wgpu::Extent3d) -> u32 {
	let sizes = [size.width, size.height, size.depth_or_array_layers];
	let max_size = sizes.into_iter().max().unwrap_or(1);
	1 + (max_size as f32).log2().floor() as u32
}

pub fn generate_mipmap_2d(painter: &Painter, texture: &wgpu::Texture) {
	let mut encoder = painter
		.device
		.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("mip gen encoder"),
		});

	for base_mip_level in 1..texture.mip_level_count() {
		let src_view = texture.create_view(&wgpu::TextureViewDescriptor {
			label: None,
			dimension: Some(wgpu::TextureViewDimension::D2),
			base_mip_level: (base_mip_level - 1),
			mip_level_count: Some(1),
			..default()
		});

		let dst_view = texture.create_view(&wgpu::TextureViewDescriptor {
			label: None,
			dimension: Some(wgpu::TextureViewDimension::D2),
			base_mip_level,
			mip_level_count: Some(1),
			..default()
		});

		let pipeline = &painter.pipelines[FULL_SCREEN_TEXTURE_PIPELINE];

		let src_binding = painter
			.device
			.create_bind_group(&wgpu::BindGroupDescriptor {
				label: None,
				layout: &pipeline.pipeline.get_bind_group_layout(1),
				entries: &[wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&src_view),
				}],
			});

		{
			let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: None,
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &dst_view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
						store: StoreOp::Store,
					},
				})],
				..Default::default()
			});

			pass.set_pipeline(&pipeline.pipeline);
			pass.set_bind_group(0, &painter.bind_groups[1].bind_group, &[]);
			pass.set_bind_group(1, &src_binding, &[]);

			pass.draw(0..3, 0..1); // Assuming a fullscreen quad
		}
	}

	painter.queue.submit(Some(encoder.finish()));
}

pub(crate) fn map_format_to_u8(format: wgpu::TextureFormat) -> u8 {
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
