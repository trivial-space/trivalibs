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
  out.coord = vec2f(f32((vertex_index << 1) & 2), f32(vertex_index & 2));
  out.position = vec4f(out.coord * 2.0 - 1.0, 0.0, 1.0);
  return out;
}

@group(0) @binding(0) var ourTexture: texture_2d<f32>;
@group(0) @binding(1) var ourSampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
	var coord = vec2f(in.coord.x, 1.0 - in.coord.y);
	return textureSample(ourTexture, ourSampler, coord);
}
"#;
