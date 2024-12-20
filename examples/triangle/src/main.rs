use shader::Vertex;
use trivalibs::painter::{
	create_canvas_app,
	wgpu::{self, include_spirv, util::DeviceExt},
	CanvasApp, Event, Painter,
};
use trivalibs::prelude::*;

struct ViewState {
	pipeline: wgpu::RenderPipeline,
	buffer: wgpu::Buffer,
	diffuse_bind_group: wgpu::BindGroup,
}

const VERTICES: &[Vertex] = &[
	Vertex {
		position: vec3(0.0, 0.5, 0.0),
		color: vec3(1.0, 0.0, 0.0),
		uv: vec2(0.5, 1.0),
	},
	Vertex {
		position: vec3(-0.5, -0.5, 0.0),
		color: vec3(0.0, 1.0, 0.0),
		uv: vec2(0.0, 0.0),
	},
	Vertex {
		position: vec3(0.5, -0.5, 0.0),
		color: vec3(0.0, 0.0, 1.0),
		uv: vec2(1.0, 0.0),
	},
];

#[derive(Default)]
struct App {}

impl CanvasApp<ViewState, ()> for App {
	fn init(&self, painter: &mut Painter) -> ViewState {
		// Initialize the app

		let buffer = painter
			.device
			.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some("Vertex Buffer"),
				contents: bytemuck::cast_slice(VERTICES),
				usage: wgpu::BufferUsages::VERTEX,
			});

		let tex_bytes = include_bytes!("../texture.png");
		let mut reader = png::Decoder::new(std::io::Cursor::new(tex_bytes))
			.read_info()
			.unwrap();
		// Allocate the output buffer.
		let mut buf = vec![0; reader.output_buffer_size()];
		// Read the next frame. An APNG might contain multiple frames.
		let info = reader.next_frame(&mut buf).unwrap();
		// Grab the bytes of the image.
		let tex_rgba = &buf[..info.buffer_size()];
		let dimensions = (info.width, info.height);

		let texture_size = wgpu::Extent3d {
			width: dimensions.0,
			height: dimensions.1,
			depth_or_array_layers: 1,
		};

		let diffuse_texture = painter.device.create_texture(&wgpu::TextureDescriptor {
			// All textures are stored as 3D, we represent our 2D texture
			// by setting depth to 1.
			size: texture_size,
			mip_level_count: 1, // We'll talk about this a little later
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			// Most images are stored using sRGB, so we need to reflect that here.
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			// TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
			// COPY_DST means that we want to copy data to this texture
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			label: Some("diffuse_texture"),
			// This is the same as with the SurfaceConfig. It
			// specifies what texture formats can be used to
			// create TextureViews for this texture. The base
			// texture format (Rgba8UnormSrgb in this case) is
			// always supported. Note that using a different
			// texture format is not supported on the WebGL2
			// backend.
			view_formats: &[],
		});

		painter.queue.write_texture(
			// Tells wgpu where to copy the pixel data
			wgpu::ImageCopyTexture {
				texture: &diffuse_texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			// The actual pixel data
			&tex_rgba,
			// The layout of the texture
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4 * dimensions.0),
				rows_per_image: Some(dimensions.1),
			},
			texture_size,
		);

		// We don't need to configure the texture view much, so let's
		// let wgpu define it.
		let diffuse_texture_view =
			diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
		let diffuse_sampler = painter.device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		let texture_bind_group_layout =
			painter
				.device
				.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					entries: &[
						wgpu::BindGroupLayoutEntry {
							binding: 0,
							visibility: wgpu::ShaderStages::FRAGMENT,
							ty: wgpu::BindingType::Texture {
								multisampled: false,
								view_dimension: wgpu::TextureViewDimension::D2,
								sample_type: wgpu::TextureSampleType::Float { filterable: true },
							},
							count: None,
						},
						wgpu::BindGroupLayoutEntry {
							binding: 1,
							visibility: wgpu::ShaderStages::FRAGMENT,
							// This should match the filterable field of the
							// corresponding Texture entry above.
							ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
							count: None,
						},
					],
					label: Some("texture_bind_group_layout"),
				});

		let diffuse_bind_group = painter
			.device
			.create_bind_group(&wgpu::BindGroupDescriptor {
				layout: &texture_bind_group_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
					},
				],
				label: Some("diffuse_bind_group"),
			});

		let pipeline_layout =
			painter
				.device
				.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
					label: None,
					bind_group_layouts: &[&texture_bind_group_layout],
					push_constant_ranges: &[],
				});

		// Load the shaders from disk
		let vert_shader = painter
			.device
			.create_shader_module(include_spirv!("../shader/vertex.spv"));
		let frag_shader = painter
			.device
			.create_shader_module(include_spirv!("../shader/fragment.spv"));

		let pipeline = painter
			.device
			.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				label: None,
				layout: Some(&pipeline_layout),
				vertex: wgpu::VertexState {
					module: &vert_shader,
					entry_point: None,
					buffers: &[wgpu::VertexBufferLayout {
						array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
						step_mode: wgpu::VertexStepMode::Vertex,
						attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2],
					}],
					compilation_options: Default::default(),
				},
				fragment: Some(wgpu::FragmentState {
					module: &frag_shader,
					entry_point: None,
					compilation_options: Default::default(),
					targets: &[Some(wgpu::ColorTargetState {
						format: painter.config.format, // for direct rendering into te surface
						blend: Some(wgpu::BlendState::REPLACE),
						write_mask: wgpu::ColorWrites::ALL,
					})],
				}),
				primitive: Default::default(),
				depth_stencil: None,
				multisample: Default::default(),
				multiview: None,
				cache: None,
			});

		ViewState {
			pipeline,
			buffer,
			diffuse_bind_group,
		}
	}

	fn render(&self, painter: &mut Painter, state: &ViewState) -> Result<(), wgpu::SurfaceError> {
		let frame = painter.surface.get_current_texture()?;

		let view = frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = painter
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
		{
			let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: None,
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
			});
			rpass.set_pipeline(&state.pipeline);
			rpass.set_bind_group(0, &state.diffuse_bind_group, &[]);
			rpass.set_vertex_buffer(0, state.buffer.slice(..));
			rpass.draw(0..3, 0..1);
		}

		painter.queue.submit(Some(encoder.finish()));
		frame.present();

		Ok(())
	}

	fn event(&mut self, _e: Event<()>, _p: &Painter) {}
	fn resize(&mut self, _p: &mut Painter, _r: &mut ViewState) {}
	fn update(&mut self, _p: &mut Painter, _r: &mut ViewState, _tpf: f32) {}
}

pub fn main() {
	create_canvas_app(App::default()).start();
}
