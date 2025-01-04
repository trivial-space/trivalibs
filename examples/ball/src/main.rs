use geom::create_ball_geom;
use trivalibs::{
	math::transform::Transform,
	painter::{
		layer::{Layer, LayerProps},
		load_fragment_shader, load_vertex_shader,
		shade::ShadeProps,
		sketch::SketchProps,
		texture::Texture2DProps,
		uniform::{Mat3U, UniformBuffer},
		wgpu::{self, VertexFormat::*},
		AppConfig, CanvasApp, Event, Painter,
	},
	prelude::*,
	rendering::{
		camera::{CamProps, PerspectiveCamera},
		scene::SceneObject,
	},
};

mod geom;

struct App {
	cam: PerspectiveCamera,
	ball_transform: Transform,

	canvas: Layer,
	mvp: UniformBuffer<Mat4>,
	norm: UniformBuffer<Mat3U>,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
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

		let texture = p.texture_2d_create(Texture2DProps {
			width: info.width,
			height: info.height,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
		});

		texture.fill_2d(p, tex_rgba);

		let uniform_type = p.uniform_type_buffered_vert();
		let tex_type = p.uniform_type_tex_2d_frag();

		let shade = p.shade_create(ShadeProps {
			vertex_format: &[Float32x3, Float32x2, Float32x3, Float32x3],
			uniform_types: &[uniform_type, uniform_type, tex_type],
		});
		load_vertex_shader!(shade, p, "../shader/vertex.spv");
		load_fragment_shader!(shade, p, "../shader/fragment.spv");

		let form = p.form_create(&create_ball_geom(), default());

		let tex = tex_type.const_tex2d(p, texture, p.sampler_default());
		let mvp = uniform_type.create_mat4(p);

		let norm = uniform_type.create_mat3(p);

		let sketch = p.sketch_create(
			form,
			shade,
			SketchProps {
				uniforms: vec![(0, mvp.uniform), (1, norm.uniform), (2, tex)],
				cull_mode: Some(wgpu::Face::Back),
				..default()
			},
		);

		let canvas = p.layer_create(LayerProps {
			clear_color: Some(wgpu::Color {
				r: 0.5,
				g: 0.6,
				b: 0.7,
				a: 1.0,
			}),
			sketches: vec![sketch],
			..default()
		});

		Self {
			canvas,
			mvp,
			norm,

			cam: PerspectiveCamera::create(CamProps {
				fov: Some(0.65),
				translation: Some(vec3(0.0, 5.0, 0.0)),
				rot_vertical: Some(-0.26),
				..default()
			}),
			ball_transform: Transform::from_translation(vec3(0.0, 0.0, -20.0)),
		}
	}

	fn resize(&mut self, _p: &mut Painter, width: u32, height: u32) {
		self.cam.set_aspect_ratio(width as f32 / height as f32);
	}

	fn update(&mut self, p: &mut Painter, tpf: f32) {
		self.ball_transform.rotate_y(tpf * 0.5);

		self.mvp
			.update(p, self.ball_transform.model_view_proj_mat(&self.cam));

		self.norm
			.update_mat3(p, self.ball_transform.view_normal_mat(&self.cam));

		p.request_next_frame();
	}

	fn render(&self, p: &mut Painter) -> Result<(), wgpu::SurfaceError> {
		p.paint(self.canvas)?;
		p.show(self.canvas)
	}

	fn event(&mut self, _e: Event<()>, _p: &mut Painter) {}
}

pub fn main() {
	App::create().config(AppConfig { show_fps: true }).start();
}
