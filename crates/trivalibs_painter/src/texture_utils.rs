// /**
//  * Given a GPUExtent3D returns the number of mip levels needed
//  *
//  * @param size
//  * @returns number of mip levels needed for the given size
//  */
// export function numMipLevels(size: GPUExtent3D, dimension?: GPUTextureDimension): number {
//    const sizes = normalizeGPUExtent3D(size);
//    const maxSize = Math.max(...sizes.slice(0, dimension === '3d' ? 3 : 2));
//    return 1 + Math.log2(maxSize) | 0;
// }

use trivalibs_core::utils::default;
use wgpu::StoreOp;

use crate::{painter::FULL_SCREEN_TEXTURE_PIPELINE, Painter};

///
pub fn num_mip_levels(size: wgpu::Extent3d) -> u32 {
	let sizes = [size.width, size.height, size.depth_or_array_layers];
	let max_size = sizes.into_iter().max().unwrap_or(1);
	1 + (max_size as f32).log2().floor() as u32
}

// // Use a WeakMap so the device can be destroyed and/or lost
// const byDevice = new WeakMap();

// /**
//  * Generates mip levels from level 0 to the last mip for an existing texture
//  *
//  * The texture must have been created with TEXTURE_BINDING and RENDER_ATTACHMENT
//  * and been created with mip levels
//  *
//  * @param device A GPUDevice
//  * @param texture The texture to create mips for
//  * @param textureBindingViewDimension This is only needed in compatibility mode
//  *   and it is only needed when the texture is going to be used as a cube map.
//  */
// export function generateMipmap(
//     device: GPUDevice,
//     texture: GPUTexture,
//     textureBindingViewDimension?: GPUTextureViewDimension) {

//   for (let baseMipLevel = 1; baseMipLevel < texture.mipLevelCount; ++baseMipLevel) {
//     for (let baseArrayLayer = 0; baseArrayLayer < texture.depthOrArrayLayers; ++baseArrayLayer) {
//       const bindGroup = device.createBindGroup({
//         layout: pipeline.getBindGroupLayout(0),
//         entries: [
//           { binding: 0, resource: sampler },
//           {
//             binding: 1,
//             resource: texture.createView({
//               dimension: textureBindingViewDimension,
//               baseMipLevel: baseMipLevel - 1,
//               mipLevelCount: 1,
//             }),
//           },
//         ],
//       });

//       const renderPassDescriptor: GPURenderPassDescriptor = {
//         label: 'mip gen renderPass',
//         colorAttachments: [
//           {
//             view: texture.createView({
//                dimension: '2d',
//                baseMipLevel,
//                mipLevelCount: 1,
//                baseArrayLayer,
//                arrayLayerCount: 1,
//             }),
//             loadOp: 'clear',
//             storeOp: 'store',
//           },
//         ],
//       };

//       const pass = encoder.beginRenderPass(renderPassDescriptor);
//       pass.setPipeline(pipeline);
//       pass.setBindGroup(0, bindGroup);
//       pass.draw(3, 1, 0, baseArrayLayer);
//       pass.end();
//     }
//   }

//   const commandBuffer = encoder.finish();
//   device.queue.submit([commandBuffer]);
// }

pub fn generate_mipmap_2d(painter: &mut Painter, texture: &wgpu::Texture) {
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
