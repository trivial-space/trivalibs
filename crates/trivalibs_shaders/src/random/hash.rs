use spirv_std::glam::UVec2;

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
