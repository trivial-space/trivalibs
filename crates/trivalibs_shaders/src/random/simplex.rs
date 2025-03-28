//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket, Johan Helsing

// Ported from https://github.com/johanhelsing/noisy_bevy from WGSL To Rust-GPU

use spirv_std::glam::{
	vec2, vec3, vec4, Vec2, Vec2Swizzles, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles,
};

fn permute_3(x: Vec3) -> Vec3 {
	((x * 34.0) + 1.0) * x % Vec3::splat(289.0)
}

pub fn simplex_noise_2d(v: Vec2) -> f32 {
	let c = vec4(
		0.211324865405187,  // (3.0 - sqrt(3.0)) / 6.0
		0.366025403784439,  // 0.5 * (sqrt(3.0) - 1.0)
		-0.577350269189626, // -1.0 + 2.0 * C.x
		0.024390243902439,  // 1.0 / 41.0
	);

	// first corner
	let i = (v + v.dot(c.yy())).floor();
	let x0 = v - i + i.dot(c.xx());

	// other corners
	let i1 = if x0.x > x0.y {
		vec2(1.0, 0.0)
	} else {
		vec2(0.0, 1.0)
	};
	let x12 = x0.xyxy() + c.xxzz() - vec4(i1.x, i1.y, 0.0, 0.0);

	// permutations
	let i = i % Vec2::splat(289.0);

	let p = permute_3(permute_3(i.y + vec3(0.0, i1.y, 1.0)) + i.x + vec3(0.0, i1.x, 1.0));
	let mut m = (Vec3::splat(0.5)
		- vec3(x0.dot(x0), x12.xy().dot(x12.xy()), x12.zw().dot(x12.zw())))
	.max(Vec3::ZERO);
	m *= m;
	m *= m;

	// gradients: 41 points uniformly over a line, mapped onto a diamond
	// the ring size, 17*17 = 289, is close to a multiple of 41 (41*7 = 287)
	let x = 2.0 * (p * c.www()).fract() - 1.0;
	let h = x.abs() - 0.5;
	let ox = (x + 0.5).floor();
	let a0 = x - ox;

	// normalize gradients implicitly by scaling m
	// approximation of: m *= inversesqrt(a0 * a0 + h * h);
	m = m * (1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h));

	// compute final noise value at P
	let v = a0.yz() * x12.xz() + h.yz() * x12.yw();
	let g = vec3(a0.x * x0.x + h.x * x0.y, v.x, v.y);
	130.0 * m.dot(g)
}

pub fn simplex_noise_2d_seeded(v: Vec2, seed: f32) -> f32 {
	let c = vec4(
		0.211324865405187,  // (3.0 - sqrt(3.0)) / 6.0
		0.366025403784439,  // 0.5 * (sqrt(3.0) - 1.0)
		-0.577350269189626, // -1.0 + 2.0 * C.x
		0.024390243902439,  // 1.0 / 41.0
	);

	// first corner
	let i = (v + v.dot(c.yy())).floor();
	let x0 = v - i + i.dot(c.xx());

	// other corners
	let i1 = if x0.x > x0.y {
		vec2(1.0, 0.0)
	} else {
		vec2(0.0, 1.0)
	};
	let x12 = x0.xyxy() + c.xxzz() - vec4(i1.x, i1.y, 0.0, 0.0);

	// permutations
	let i = i % Vec2::splat(289.0);

	let mut p = permute_3(permute_3(i.y + vec3(0.0, i1.y, 1.0)) + i.x + vec3(0.0, i1.x, 1.0));
	p = permute_3(p + Vec3::splat(seed));
	let mut m = (Vec3::splat(0.5)
		- vec3(x0.dot(x0), x12.xy().dot(x12.xy()), x12.zw().dot(x12.zw())))
	.max(Vec3::splat(0.0));
	m *= m;
	m *= m;

	// gradients: 41 points uniformly over a line, mapped onto a diamond
	// the ring size, 17*17 = 289, is close to a multiple of 41 (41*7 = 287)
	let x = 2.0 * (p * c.www()).fract() - 1.0;
	let h = x.abs() - 0.5;
	let ox = (x + 0.5).floor();
	let a0 = x - ox;

	// normalize gradients implicitly by scaling m
	// approximation of: m *= inversesqrt(a0 * a0 + h * h);
	m = m * (1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h));

	// compute final noise value at P
	let v = a0.yz() * x12.xz() + h.yz() * x12.yw();
	let g = vec3(a0.x * x0.x + h.x * x0.y, v.x, v.y);
	130.0 * m.dot(g)
}

fn permute_4(x: Vec4) -> Vec4 {
	((x * 34.0) + 1.0) * x % Vec4::splat(289.0)
}

fn taylor_inv_sqrt_4(r: Vec4) -> Vec4 {
	Vec4::splat(1.79284291400159) - Vec4::splat(0.85373472095314) * r
}

pub fn simplex_noise_3d(v: Vec3) -> f32 {
	let c = vec2(1.0 / 6.0, 1.0 / 3.0);
	let d = vec4(0.0, 0.5, 1.0, 2.0);

	// first corner
	let i = (v + v.dot(c.yyy())).floor();
	let x0 = v - i + i.dot(c.xxx());

	// other corners
	let g = step_v3(x0.yzx(), x0.xyz());
	let l = Vec3::splat(1.0) - g;
	let i1 = g.min(l.zxy());
	let i2 = g.max(l.zxy());

	// x0 = x0 - 0. + 0. * C
	let x1 = x0 - i1 + c.xxx();
	let x2 = x0 - i2 + 2.0 * c.xxx();
	let x3 = x0 - Vec3::splat(1.0) + 3.0 * c.xxx();

	// permutations
	let i = i % Vec3::splat(289.0);
	let p = permute_4(
		permute_4(permute_4(i.z + vec4(0.0, i1.z, i2.z, 1.0)) + i.y + vec4(0.0, i1.y, i2.y, 1.0))
			+ i.x + vec4(0.0, i1.x, i2.x, 1.0),
	);

	// gradients (NxN points uniformly over a square, mapped onto an octahedron)
	let n_ = 1.0 / 7.0; // N=7
	let ns = n_ * d.wyz() - d.xzx();

	let j = p - 49.0 * (p * ns.z * ns.z).floor(); // mod(p, N*N)

	let x_ = (j * ns.z).floor();
	let y_ = (j - 7.0 * x_).floor(); // mod(j, N)

	let x = x_ * ns.x + ns.yyyy();
	let y = y_ * ns.x + ns.yyyy();
	let h = 1.0 - x.abs() - y.abs();

	let b0 = vec4(x.x, x.y, y.x, y.y);
	let b1 = vec4(x.z, x.w, y.z, y.w);

	let s0 = b0.floor() * 2.0 + 1.0;
	let s1 = b1.floor() * 2.0 + 1.0;
	let sh = -step_v4(h, Vec4::ZERO);

	let a0 = b0.xzyw() + s0.xzyw() * sh.xxyy();
	let a1 = b1.xzyw() + s1.xzyw() * sh.zzww();

	let mut p0 = vec3(a0.x, a0.y, h.x);
	let mut p1 = vec3(a0.z, a0.w, h.y);
	let mut p2 = vec3(a1.x, a1.y, h.z);
	let mut p3 = vec3(a1.z, a1.w, h.w);

	// normalize gradients
	let norm = taylor_inv_sqrt_4(vec4(p0.dot(p0), p1.dot(p1), p2.dot(p2), p3.dot(p3)));
	p0 *= norm.x;
	p1 *= norm.y;
	p2 *= norm.z;
	p3 *= norm.w;

	// mix final noise value
	let mut m = 0.6 - vec4(x0.dot(x0), x1.dot(x1), x2.dot(x2), x3.dot(x3));
	m = m.max(Vec4::ZERO);
	m *= m;
	m *= m;
	42.0 * m.dot(vec4(p0.dot(x0), p1.dot(x1), p2.dot(x2), p3.dot(x3)))
}

pub fn fbm_simplex_2d(pos: Vec2, octaves: i32, lacunarity: f32, gain: f32) -> f32 {
	let mut sum = 0.0;
	let mut amplitude = 1.0;
	let mut frequency = 1.0;

	for _ in 0..octaves {
		sum += simplex_noise_2d(pos * frequency) * amplitude;
		amplitude *= gain;
		frequency *= lacunarity;
	}

	sum
}

pub fn fbm_simplex_2d_seeded(
	pos: Vec2,
	octaves: i32,
	lacunarity: f32,
	gain: f32,
	seed: f32,
) -> f32 {
	let mut sum = 0.0;
	let mut amplitude = 1.0;
	let mut frequency = 1.0;

	for _ in 0..octaves {
		sum += simplex_noise_2d_seeded(pos * frequency, seed) * amplitude;
		amplitude *= gain;
		frequency *= lacunarity;
	}

	sum
}

pub fn fbm_simplex_3d(pos: Vec3, octaves: i32, lacunarity: f32, gain: f32) -> f32 {
	let mut sum = 0.0;
	let mut amplitude = 1.0;
	let mut frequency = 1.0;

	for _ in 0..octaves {
		sum += simplex_noise_3d(pos * frequency) * amplitude;
		amplitude *= gain;
		frequency *= lacunarity;
	}

	sum
}

fn step_v3(x: Vec3, y: Vec3) -> Vec3 {
	vec3(
		if x.x <= y.x { 1.0 } else { 0.0 },
		if x.y <= y.y { 1.0 } else { 0.0 },
		if x.z <= y.z { 1.0 } else { 0.0 },
	)
}

fn step_v4(x: Vec4, y: Vec4) -> Vec4 {
	vec4(
		if x.x <= y.x { 1.0 } else { 0.0 },
		if x.y <= y.y { 1.0 } else { 0.0 },
		if x.z <= y.z { 1.0 } else { 0.0 },
		if x.w <= y.w { 1.0 } else { 0.0 },
	)
}
