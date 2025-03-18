// Full screen triangle concept explained here:
// https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
pub const FULL_SCREEN_QUAD: &str = r#"
struct VertexOutput {
  @builtin(position) position: vec4f,
  @location(0) coord: vec2f,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
  var out: VertexOutput;
  var coord = vec2f(f32((vertex_index << 1) & 2), f32(vertex_index & 2));
  out.position = vec4f(coord * 2.0 - 1.0, 0.0, 1.0);
	coord.y = 1.0 - coord.y;
	out.coord = coord;
  return out;
}

@group(0) @binding(0) var ourSampler: sampler;
@group(1) @binding(0) var ourTexture: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
	return textureSample(ourTexture, ourSampler, in.coord);
}
"#;
