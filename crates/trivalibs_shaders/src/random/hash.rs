use spirv_std::glam::{vec2, vec3, vec4, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};

fn u32_to_f32(x: u32) -> f32 {
	x as f32 / 0xffffffffu32 as f32
}

// Imported and ported from https://www.shadertoy.com/view/WttXWX

// // --- from Chris Wellons https://nullprogram.com/blog/2018/07/31/
// https://github.com/skeeto/hash-prospector

// bias: 0.10760229515479501
// has excellent results if tested here: https://www.shadertoy.com/view/XlGcRh
pub fn hashi(x: u32) -> u32 {
	let mut x = x;
	x ^= x >> 16;
	x = x.wrapping_mul(0x21f0aaad);
	x ^= x >> 15;
	x = x.wrapping_mul(0xd35a2d97);
	x ^ (x >> 15)
}

// bias: 0.020888578919738908 = minimal theoretic limit
// probably hashi is good enough for most cases
pub fn hashi_triple32(x: u32) -> u32 {
	let mut x = x;
	x ^= x >> 17;
	x = x.wrapping_mul(0xed5ad4bb);
	x ^= x >> 11;
	x = x.wrapping_mul(0xac4c1b51);
	x ^= x >> 15;
	x = x.wrapping_mul(0x31848bab);
	x ^ (x >> 14)
}

pub fn hash(x: u32) -> f32 {
	u32_to_f32(hashi(x))
}

// // The MIT License
// // Copyright © 2017,2024 Inigo Quilez
// // Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions: The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

// ported from https://www.shadertoy.com/view/4tXyWN by Inigo Quilez

pub fn hash21i(p: UVec2) -> u32 {
	let mut p = p;
	p *= UVec2::new(73333, 7777);
	p = p ^ (UVec2::splat(3333777777) >> (p >> 28));
	let n = p.x * p.y;
	n ^ (n >> 15)
}

pub fn hash21(p: UVec2) -> f32 {
	u32_to_f32(hash21i(p))
}

// // https://www.pcg-random.org/
// see https://www.shadertoy.com/view/XlGcRh

// uvec2 pcg2d(uvec2 v)
// {
//     v = v * 1664525u + 1013904223u;

//     v.x += v.y * 1664525u;
//     v.y += v.x * 1664525u;

//     v = v ^ (v>>16u);

//     v.x += v.y * 1664525u;
//     v.y += v.x * 1664525u;

//     v = v ^ (v>>16u);

//     return v;
// }
pub fn hash2di(v: UVec2) -> UVec2 {
	let mut v = v;
	v = v * UVec2::new(1664525, 1013904223);
	v.x += v.y * 1664525;
	v.y += v.x * 1664525;

	v = v ^ (v >> 16);

	v.x += v.y * 1664525;
	v.y += v.x * 1664525;

	v = v ^ (v >> 16);

	v
}

pub fn hash2d(v: UVec2) -> Vec2 {
	let hash = hash2di(v);
	vec2(u32_to_f32(hash.x), u32_to_f32(hash.y))
}

// // http://www.jcgt.org/published/0009/03/02/
// uvec3 pcg3d(uvec3 v) {

//     v = v * 1664525u + 1013904223u;

//     v.x += v.y*v.z;
//     v.y += v.z*v.x;
//     v.z += v.x*v.y;

//     v ^= v >> 16u;

//     v.x += v.y*v.z;
//     v.y += v.z*v.x;
//     v.z += v.x*v.y;

//     return v;
// }

pub fn hash3di(v: UVec3) -> UVec3 {
	let mut v = v;
	v = v * UVec3::new(1664525, 1013904223, 1013904223);
	v.x += v.y * v.z;
	v.y += v.z * v.x;
	v.z += v.x * v.y;

	v = v ^ (v >> 16);

	v.x += v.y * v.z;
	v.y += v.z * v.x;
	v.z += v.x * v.y;

	v
}

pub fn hash3d(v: UVec3) -> Vec3 {
	let hash = hash3di(v);
	vec3(u32_to_f32(hash.x), u32_to_f32(hash.y), u32_to_f32(hash.z))
}

// // http://www.jcgt.org/published/0009/03/02/
// uvec4 pcg4d(uvec4 v)
// {
//     v = v * 1664525u + 1013904223u;

//     v.x += v.y*v.w;
//     v.y += v.z*v.x;
//     v.z += v.x*v.y;
//     v.w += v.y*v.z;

//     v ^= v >> 16u;

//     v.x += v.y*v.w;
//     v.y += v.z*v.x;
//     v.z += v.x*v.y;
//     v.w += v.y*v.z;

//     return v;
// }

pub fn hash4di(v: UVec4) -> UVec4 {
	let mut v = v;
	v = v * UVec4::new(1664525, 1013904223, 1013904223, 1013904223);
	v.x += v.y * v.w;
	v.y += v.z * v.x;
	v.z += v.x * v.y;
	v.w += v.y * v.z;

	v = v ^ (v >> 16);

	v.x += v.y * v.w;
	v.y += v.z * v.x;
	v.z += v.x * v.y;
	v.w += v.y * v.z;

	v
}

pub fn hash4d(v: UVec4) -> Vec4 {
	let hash = hash4di(v);
	vec4(
		u32_to_f32(hash.x),
		u32_to_f32(hash.y),
		u32_to_f32(hash.z),
		u32_to_f32(hash.w),
	)
}
