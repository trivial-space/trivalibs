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
			pass.set_bind_group(0, &painter.bindings[1].binding, &[]);
			pass.set_bind_group(1, &src_binding, &[]);

			pass.draw(0..3, 0..1); // Assuming a fullscreen quad
		}
	}

	painter.queue.submit(Some(encoder.finish()));
}
