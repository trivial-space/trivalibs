use std::io::Write;
use trivalibs::painter::wgpu::{self, include_spirv};

fn output_image_native(image_data: Vec<u8>, texture_dims: (usize, usize), path: String) {
	let mut png_data = Vec::<u8>::with_capacity(image_data.len());
	let mut encoder = png::Encoder::new(
		std::io::Cursor::new(&mut png_data),
		texture_dims.0 as u32,
		texture_dims.1 as u32,
	);
	encoder.set_color(png::ColorType::Rgba);
	let mut png_writer = encoder.write_header().unwrap();
	png_writer.write_image_data(&image_data[..]).unwrap();
	png_writer.finish().unwrap();
	log::info!("PNG file encoded in memory.");

	let mut file = std::fs::File::create(&path).unwrap();
	file.write_all(&png_data[..]).unwrap();
	log::info!("PNG file written to disc as \"{}\".", path);
}

const TEXTURE_DIMS: (usize, usize) = (512, 512);

async fn run(_path: Option<String>) {
	// This will later store the raw pixel value data locally. We'll create it now as
	// a convenient size reference.
	let mut texture_data = Vec::<u8>::with_capacity(TEXTURE_DIMS.0 * TEXTURE_DIMS.1 * 4);

	let instance = wgpu::Instance::default();
	let adapter = instance
		.request_adapter(&wgpu::RequestAdapterOptions::default())
		.await
		.unwrap();
	let (device, queue) = adapter
		.request_device(
			&wgpu::DeviceDescriptor {
				label: None,
				required_features: wgpu::Features::empty(),
				required_limits: wgpu::Limits::downlevel_defaults(),
				memory_hints: wgpu::MemoryHints::MemoryUsage,
			},
			None,
		)
		.await
		.unwrap();

	// Load the shaders from disk
	let vert = device.create_shader_module(include_spirv!("../shader/main_vs.spv"));
	let frag = device.create_shader_module(include_spirv!("../shader/main_fs.spv"));

	let render_target = device.create_texture(&wgpu::TextureDescriptor {
		label: None,
		size: wgpu::Extent3d {
			width: TEXTURE_DIMS.0 as u32,
			height: TEXTURE_DIMS.1 as u32,
			depth_or_array_layers: 1,
		},
		mip_level_count: 1,
		sample_count: 1,
		dimension: wgpu::TextureDimension::D2,
		format: wgpu::TextureFormat::Rgba8UnormSrgb,
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
		view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
	});
	let output_staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
		label: None,
		size: texture_data.capacity() as u64,
		usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
		mapped_at_creation: false,
	});

	let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: None,
		layout: None,
		vertex: wgpu::VertexState {
			module: &vert,
			entry_point: None,
			compilation_options: Default::default(),
			buffers: &[],
		},
		fragment: Some(wgpu::FragmentState {
			module: &frag,
			entry_point: None,
			compilation_options: Default::default(),
			targets: &[Some(wgpu::TextureFormat::Rgba8UnormSrgb.into())],
		}),
		primitive: wgpu::PrimitiveState::default(),
		depth_stencil: None,
		multisample: wgpu::MultisampleState::default(),
		multiview: None,
		cache: None,
	});

	log::info!("Wgpu context set up.");

	//-----------------------------------------------

	let texture_view = render_target.create_view(&wgpu::TextureViewDescriptor::default());

	let mut command_encoder =
		device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
	{
		let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: None,
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &texture_view,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
					store: wgpu::StoreOp::Store,
				},
			})],
			depth_stencil_attachment: None,
			occlusion_query_set: None,
			timestamp_writes: None,
		});
		render_pass.set_pipeline(&pipeline);
		render_pass.draw(0..3, 0..1);
	}
	// The texture now contains our rendered image
	command_encoder.copy_texture_to_buffer(
		wgpu::ImageCopyTexture {
			texture: &render_target,
			mip_level: 0,
			origin: wgpu::Origin3d::ZERO,
			aspect: wgpu::TextureAspect::All,
		},
		wgpu::ImageCopyBuffer {
			buffer: &output_staging_buffer,
			layout: wgpu::ImageDataLayout {
				offset: 0,
				// This needs to be a multiple of 256. Normally we would need to pad
				// it but we here know it will work out anyways.
				bytes_per_row: Some((TEXTURE_DIMS.0 * 4) as u32),
				rows_per_image: Some(TEXTURE_DIMS.1 as u32),
			},
		},
		wgpu::Extent3d {
			width: TEXTURE_DIMS.0 as u32,
			height: TEXTURE_DIMS.1 as u32,
			depth_or_array_layers: 1,
		},
	);
	queue.submit(Some(command_encoder.finish()));
	log::info!("Commands submitted.");

	//-----------------------------------------------

	// Time to get our image.
	let buffer_slice = output_staging_buffer.slice(..);
	let (sender, receiver) = flume::bounded(1);
	buffer_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
	device.poll(wgpu::Maintain::wait()).panic_on_timeout();
	receiver.recv_async().await.unwrap().unwrap();
	log::info!("Output buffer mapped.");
	{
		let view = buffer_slice.get_mapped_range();
		texture_data.extend_from_slice(&view[..]);
	}
	log::info!("Image data copied to local.");
	output_staging_buffer.unmap();

	output_image_native(texture_data.to_vec(), TEXTURE_DIMS, _path.unwrap());
	log::info!("Done.");
}

pub fn main() {
	env_logger::builder()
		.filter_level(log::LevelFilter::Info)
		.format_timestamp(None)
		.init();

	let path = std::env::args()
		.nth(2)
		.unwrap_or_else(|| "output.png".to_string());

	pollster::block_on(run(Some(path)));
}
