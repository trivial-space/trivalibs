//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket, Johan Helsing

// Ported from https://github.com/johanhelsing/noisy_bevy from WGSL To Rust-GPU
// Original code at https://github.com/stegu/webgl-noise by Stefan Gustavson

use crate::{num_ext::NumExt, vec_ext::VecExt};
use core::f32::consts::TAU;
use spirv_std::glam::{
	Mat3, Vec2, Vec2Swizzles, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles, mat3, vec2, vec3, vec4,
};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

/// Permutation polynomial for pseudo-random hashing (scalar version).
fn permute_1(x: f32) -> f32 {
	((x * 34.0) + 10.0) * x % 289.0
}

/// Permutation polynomial for pseudo-random hashing (Vec3 version).
fn permute_3(x: Vec3) -> Vec3 {
	((x * 34.0) + 10.0) * x % Vec3::splat(289.0)
}

/// Permutation polynomial for pseudo-random hashing (Vec4 version).
fn permute_4(x: Vec4) -> Vec4 {
	((x * 34.0) + 10.0) * x % Vec4::splat(289.0)
}

/// Taylor series approximation of inverse square root (scalar version).
fn taylor_inv_sqrt_1(r: f32) -> f32 {
	1.79284291400159 - 0.85373472095314 * r
}

/// Taylor series approximation of inverse square root (Vec4 version).
fn taylor_inv_sqrt_4(r: Vec4) -> Vec4 {
	1.79284291400159 - 0.85373472095314 * r
}

/// Computes gradient vector for 4D simplex noise.
fn grad_4(j: f32, ip: Vec4) -> Vec4 {
	let tmp = ((j * ip.xyz()).frct() * 7.0).floor() * ip.z - 1.0;
	let mut p = vec4(tmp.x, tmp.y, tmp.z, 1.5 - tmp.abs().dot(Vec3::ONE));

	let s = Vec4::ZERO.step(p);

	let tmp = (s.xyz() * 2.0 - 1.0) * s.www();
	p.x += tmp.x;
	p.y += tmp.y;
	p.z += tmp.z;

	p
}

/// 2D simplex noise.
///
/// # Parameters
///
/// * `pos` - Input position coordinates
///
/// # Returns
///
/// Noise value in approximate range [-1.0, 1.0]
pub fn simplex_noise_2d(pos: Vec2) -> f32 {
	let c = vec4(
		0.211324865405187,  // (3.0 - sqrt(3.0)) / 6.0
		0.366025403784439,  // 0.5 * (sqrt(3.0) - 1.0)
		-0.577350269189626, // -1.0 + 2.0 * C.x
		0.024390243902439,  // 1.0 / 41.0
	);

	// first corner
	let i = (pos + pos.dot(c.yy())).floor();
	let x0 = pos - i + i.dot(c.xx());

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

/// 3D simplex noise.
///
/// # Parameters
///
/// * `pos` - Input position coordinates
///
/// # Returns
///
/// Noise value in approximate range [-1.0, 1.0]
pub fn simplex_noise_3d(pos: Vec3) -> f32 {
	let c = vec2(1.0 / 6.0, 1.0 / 3.0);
	let d = vec4(0.0, 0.5, 1.0, 2.0);

	// first corner
	let i = (pos + pos.dot(c.yyy())).floor();
	let x0 = pos - i + i.dot(c.xxx());

	// other corners
	let g = x0.xyz().step(x0.yzx());
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
	let sh = -Vec4::ZERO.step(h);

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

/// Fractal Brownian Motion using 2D simplex noise.
///
/// # Parameters
///
/// * `pos` - Input position
/// * `octaves` - Number of noise layers
/// * `freq_factor` - Frequency multiplier per octave
/// * `amplitude_factor` - Amplitude multiplier per octave
pub fn fbm_simplex_2d(pos: Vec2, octaves: i32, freq_factor: f32, amplitude_factor: f32) -> f32 {
	let mut sum = 0.0;
	let mut amplitude = 1.0;
	let mut frequency = 1.0;

	for _ in 0..octaves {
		sum += simplex_noise_2d(pos * frequency) * amplitude;
		amplitude *= amplitude_factor;
		frequency *= freq_factor;
	}

	sum
}

/// Fractal Brownian Motion using 3D simplex noise.
///
/// # Parameters
///
/// * `pos` - Input position coordinates
/// * `octaves` - Number of noise layers to combine
/// * `freq_factor` - Frequency multiplier per octave (typically 2.0)
/// * `amplitude_factor` - Amplitude multiplier per octave (typically 0.5)
///
/// # Returns
///
/// Combined noise value. Range depends on octaves and amplitude_factor parameters.
pub fn fbm_simplex_3d(pos: Vec3, octaves: i32, freq_factor: f32, amplitude_factor: f32) -> f32 {
	let mut sum = 0.0;
	let mut amplitude = 1.0;
	let mut frequency = 1.0;

	for _ in 0..octaves {
		sum += simplex_noise_3d(pos * frequency) * amplitude;
		amplitude *= amplitude_factor;
		frequency *= freq_factor;
	}

	sum
}

/// 4D simplex noise.
///
/// # Parameters
///
/// * `pos` - Input position coordinates (4D)
///
/// # Returns
///
/// Noise value in approximate range [-1.0, 1.0]
pub fn simplex_noise_4d(pos: Vec4) -> f32 {
	let c = vec4(
		0.138196601125011,  // (5 - sqrt(5))/20  G4
		0.276393202250021,  // 2 * G4
		0.414589803375032,  // 3 * G4
		-0.447213595499958, // -1 + 4 * G4
	);

	// First corner
	let i = (pos + pos.dot(Vec4::splat(0.309016994374947451))).floor(); // (sqrt(5) - 1)/4
	let x0 = pos - i + i.dot(c.xxxx());

	// Other corners
	let is_x = x0.xxx().step(x0.yzw());
	let is_yz = x0.yyz().step(x0.zww());

	// Rank sorting originally contributed by Bill Licea-Kane, AMD (formerly ATI)
	let tmp = Vec3::ONE - is_x;
	let mut i0 = vec4(is_x.x + is_x.y + is_x.z, tmp.x, tmp.y, tmp.z);
	i0.y += is_yz.x + is_yz.y;

	let tmp = Vec2::ONE - is_yz.xy();
	i0.z += tmp.x + is_yz.z;
	i0.w += tmp.y + 1.0 - is_yz.z;

	// i0 now contains the unique values 0,1,2,3 in each channel
	let i3 = i0.clamp01();
	let i2 = (i0 - Vec4::ONE).clamp01();
	let i1 = (i0 - Vec4::splat(2.0)).clamp01();

	let x1 = x0 - i1 + c.xxxx();
	let x2 = x0 - i2 + c.yyyy();
	let x3 = x0 - i3 + c.zzzz();
	let x4 = x0 + c.wwww();

	// Permutations
	let i = i % Vec4::splat(289.0);
	let j0 = permute_1(permute_1(permute_1(permute_1(i.w) + i.z) + i.y) + i.x);
	let j1 = permute_4(
		permute_4(
			permute_4(
				permute_4(i.w + vec4(i1.w, i2.w, i3.w, 1.0)) + i.z + vec4(i1.z, i2.z, i3.z, 1.0),
			) + i.y + vec4(i1.y, i2.y, i3.y, 1.0),
		) + i.x + vec4(i1.x, i2.x, i3.x, 1.0),
	);

	// Gradients: 7x7x6 points over a cube, mapped onto a 4-cross polytope
	// 7*7*6 = 294, which is close to the ring size 17*17 = 289.
	let ip = vec4(1.0 / 294.0, 1.0 / 49.0, 1.0 / 7.0, 0.0);

	let p0 = grad_4(j0, ip);
	let p1 = grad_4(j1.x, ip);
	let p2 = grad_4(j1.y, ip);
	let p3 = grad_4(j1.z, ip);
	let p4 = grad_4(j1.w, ip);

	// Normalize gradients
	let norm = taylor_inv_sqrt_4(vec4(p0.dot(p0), p1.dot(p1), p2.dot(p2), p3.dot(p3)));
	let p0 = p0 * norm.x;
	let p1 = p1 * norm.y;
	let p2 = p2 * norm.z;
	let p3 = p3 * norm.w;
	let p4 = p4 * taylor_inv_sqrt_1(p4.dot(p4));

	// Mix contributions from the five corners
	let mut m0 = vec3(0.6 - x0.dot(x0), 0.6 - x1.dot(x1), 0.6 - x2.dot(x2)).max(Vec3::ZERO);
	let mut m1 = vec2(0.6 - x3.dot(x3), 0.6 - x4.dot(x4)).max(Vec2::ZERO);

	m0 *= m0;
	m1 *= m1;

	49.0 * (m0.dot(m0 * vec3(p0.dot(x0), p1.dot(x1), p2.dot(x2)))
		+ m1.dot(m1 * vec2(p3.dot(x3), p4.dot(x4))))
}

/// Seamlessly tiling 2D simplex noise using 4D noise mapped to a torus.
///
/// # Parameters
///
/// * `pos` - Input position coordinates (should be in range [0.0, 1.0] for seamless tiling)
/// * `scale` - Scale factor for the noise
///
/// # Returns
///
/// Noise value in approximate range [-1.0, 1.0]
pub fn tiling_simplex_noise_2d(pos: Vec2, scale: f32) -> f32 {
	// Map coordinates to circle for seamless wrapping
	let angle_x = pos.x * TAU;
	let angle_y = pos.y * TAU;

	let nx = angle_x.cos() * scale;
	let ny = angle_x.sin() * scale;
	let nz = angle_y.cos() * scale;
	let nw = angle_y.sin() * scale;

	// Get 4D noise value
	simplex_noise_4d(vec4(nx, ny, nz, nw))
}

/// 2D tiling simplex noise with rotating gradients and analytical derivative.
///
/// Implementation based on psrdnoise by Stefan Gustavson and Ian McEwan.
/// See <https://github.com/stegu/psrdnoise/>
///
/// # Parameters
///
/// * `pos` - Point (x,y) to evaluate
/// * `period` - Desired periods along x and y. Set to 0.0 or negative to skip wrapping along that dimension.
/// * `norm_rot` - Normalized rotation for swirling gradients, where 1.0 is TAU (2π radians)
///
/// # Returns
///
/// A tuple containing:
/// * Noise value in approximate range [-1.0, 1.0]
/// * Gradient vector (x,y partial derivatives)
///
/// # Performance Notes
///
/// * Setting both periods to 0.0 makes the function execute ~15% faster
/// * Not using the gradient return value allows compiler optimization for 10-15% speedup
pub fn tiling_rot_noise_2d(pos: Vec2, period: Vec2, norm_rot: f32) -> (f32, Vec2) {
	// Transform to simplex space (axis-aligned hexagonal grid)
	let uv = vec2(pos.x + pos.y * 0.5, pos.y);

	// Determine which simplex we're in, with i0 being the "base"
	let i0 = uv.floor();
	let f0 = uv.fract();
	// o1 is the offset in simplex space to the second corner
	let cmp = f0.x.step(f0.y);
	let o1 = vec2(cmp, 1.0 - cmp);

	// Enumerate the remaining simplex corners
	let i1 = i0 + o1;
	let i2 = i0 + vec2(1.0, 1.0);

	// Transform corners back to texture space
	let v0 = vec2(i0.x - i0.y * 0.5, i0.y);
	let v1 = vec2(v0.x + o1.x - o1.y * 0.5, v0.y + o1.y);
	let v2 = vec2(v0.x + 0.5, v0.y + 1.0);

	// Compute vectors from v to each of the simplex corners
	let x0 = pos - v0;
	let x1 = pos - v1;
	let x2 = pos - v2;

	// Wrap to periods, if desired
	let (iu, iv) = if period.x > 0.0 || period.y > 0.0 {
		let xw = vec3(v0.x, v1.x, v2.x);
		let yw = vec3(v0.y, v1.y, v2.y);
		let xw = if period.x > 0.0 { xw % period.x } else { xw };
		let yw = if period.y > 0.0 { yw % period.y } else { yw };
		((xw + 0.5 * yw + 0.5).floor(), (yw + 0.5).floor())
	} else {
		(vec3(i0.x, i1.x, i2.x), vec3(i0.y, i1.y, i2.y))
	};

	// Compute one pseudo-random hash value for each corner
	let hash = iu % Vec3::splat(289.);
	let hash = ((hash * 51.0 + 2.0) * hash + iv) % Vec3::splat(289.);
	let hash = ((hash * 34.0 + 10.0) * hash) % Vec3::splat(289.);

	// Pick a pseudo-random angle and add the desired rotation
	let psi = hash * 0.07482 + (norm_rot * TAU);
	let gx = psi.cos();
	let gy = psi.sin();

	// Reorganize for dot products below
	let g0 = vec2(gx.x, gy.x);
	let g1 = vec2(gx.y, gy.y);
	let g2 = vec2(gx.z, gy.z);

	// Radial decay with distance from each simplex corner
	let w = vec3(0.8 - x0.dot(x0), 0.8 - x1.dot(x1), 0.8 - x2.dot(x2)).max(Vec3::ZERO);
	let w2 = w * w;
	let w4 = w2 * w2;

	// The value of the linear ramp from each of the corners
	let gdotx = vec3(g0.dot(x0), g1.dot(x1), g2.dot(x2));

	// Multiply by the radial decay and sum up the noise value
	let n = w4.dot(gdotx);

	// Compute the first order partial derivatives
	let w3 = w2 * w;
	let dw = -8.0 * w3 * gdotx;
	let dn0 = w4.x * g0 + dw.x * x0;
	let dn1 = w4.y * g1 + dw.y * x1;
	let dn2 = w4.z * g2 + dw.z * x2;

	// Scale the return value to fit nicely into the range [-1, 1]
	(10.9 * n, 10.9 * (dn0 + dn1 + dn2))
}

/// 2D tiling simplex noise with fixed gradients and analytical derivative.
///
/// Wrapper around `tiling_rot_noise_2d` with no rotation.
///
/// # Parameters
///
/// * `pos` - Point (x,y) to evaluate
/// * `period` - Desired periods along x and y. Set to 0.0 or negative to skip wrapping along that dimension.
///
/// # Returns
///
/// A tuple containing:
/// * Noise value in approximate range [-1.0, 1.0]
/// * Gradient vector (x,y partial derivatives)
pub fn tiling_noise_2d(pos: Vec2, period: Vec2) -> (f32, Vec2) {
	tiling_rot_noise_2d(pos, period, 0.0)
}

/// 2D simplex noise with rotating gradients and analytical derivative (no tiling).
///
/// Wrapper around `tiling_rot_noise_2d` with no period wrapping.
///
/// # Parameters
///
/// * `pos` - Point (x,y) to evaluate
/// * `norm_rot` - Normalized rotation for swirling gradients, where 1.0 is TAU (2π radians)
///
/// # Returns
///
/// A tuple containing:
/// * Noise value in approximate range [-1.0, 1.0]
/// * Gradient vector (x,y partial derivatives)
pub fn rot_noise_2d(pos: Vec2, norm_rot: f32) -> (f32, Vec2) {
	tiling_rot_noise_2d(pos, Vec2::ZERO, norm_rot)
}

/// 3D tiling simplex noise with rotating gradients and analytical derivatives.
///
/// Implementation based on psrdnoise by Stefan Gustavson and Ian McEwan.
/// See <https://github.com/stegu/psrdnoise/>
///
/// This implementation uses an axis-aligned grid to permit rectangular tiling.
/// The noise pattern can tile seamlessly to any integer periods up to 289 units
/// in the x, y and z directions. The rotating gradients create a swirling motion
/// effect, useful for animation without needing a 4th dimension.
///
/// # Parameters
///
/// * `pos` - Point (x,y,z) to evaluate
/// * `period` - Desired periods along x,y,z (up to 289). Set to 0.0 or negative to skip wrapping along that dimension.
/// * `norm_rot` - Normalized rotation for swirling gradients, where 1.0 is TAU (2π radians)
///
/// # Returns
///
/// A tuple containing:
/// * Noise value in approximate range [-1.0, 1.0]
/// * Gradient vector (x,y,z partial derivatives)
///
/// # Performance Notes
///
/// * Function executes 15-20% faster if `norm_rot` is constant 0.0
/// * Setting all periods to 0.0 makes the function execute 10-15% faster
/// * Not using the gradient return value allows compiler optimization for ~10% speedup
pub fn tiling_rot_noise_3d(pos: Vec3, period: Vec3, norm_rot: f32) -> (f32, Vec3) {
	const M: Mat3 = mat3(
		vec3(0.0, 1.0, 1.0),
		vec3(1.0, 0.0, 1.0),
		vec3(1.0, 1.0, 0.0),
	);
	const MI: Mat3 = mat3(
		vec3(-0.5, 0.5, 0.5),
		vec3(0.5, -0.5, 0.5),
		vec3(0.5, 0.5, -0.5),
	);

	let uvw = M * pos; // Transform to simplex space

	// Determine which simplex we're in, with i0 being the "base corner"
	let mut i0 = uvw.floor();
	let f0 = uvw.fract(); // coords within "skewed cube"

	// To determine which simplex corners are closest, rank order the
	// magnitudes of u,v,w, resolving ties in priority order u,v,w,
	// and traverse the four corners from largest to smallest magnitude.
	// o1, o2 are offsets in simplex space to the 2nd and 3rd corners.
	let g_ = f0.yzz().step(f0.xyx()); // Makes comparison "less-than"
	let l_ = 1.0 - g_; // complement is "greater-or-equal"
	let g = vec3(l_.z, g_.x, g_.y);
	let l = vec3(l_.x, l_.y, g_.z);
	let o1 = g.min(l);
	let o2 = g.max(l);

	// Enumerate the remaining simplex corners
	let mut i1 = i0 + o1;
	let mut i2 = i0 + o2;
	let mut i3 = i0 + 1.0;

	// Transform corners back to texture space
	let v0 = MI * i0;
	let v1 = MI * i1;
	let v2 = MI * i2;
	let v3 = MI * i3;

	// Compute vectors from v to each of the simplex corners
	let x0 = pos - v0;
	let x1 = pos - v1;
	let x2 = pos - v2;
	let x3 = pos - v3;

	if period.x > 0.0 || period.y > 0.0 || period.z > 0.0 {
		// Wrap to periods and transform back to simplex space
		let vx = vec4(v0.x, v1.x, v2.x, v3.x);
		let vy = vec4(v0.y, v1.y, v2.y, v3.y);
		let vz = vec4(v0.z, v1.z, v2.z, v3.z);

		// Wrap to periods where specified
		let vx = if period.x > 0.0 { vx % period.x } else { vx };
		let vy = if period.y > 0.0 { vy % period.y } else { vy };
		let vz = if period.z > 0.0 { vz % period.z } else { vz };

		// Transform wrapped coordinates back to uvw
		i0 = M * vec3(vx.x, vy.x, vz.x);
		i1 = M * vec3(vx.y, vy.y, vz.y);
		i2 = M * vec3(vx.z, vy.z, vz.z);
		i3 = M * vec3(vx.w, vy.w, vz.w);

		// Fix rounding errors
		i0 = (i0 + 0.5).floor();
		i1 = (i1 + 0.5).floor();
		i2 = (i2 + 0.5).floor();
		i3 = (i3 + 0.5).floor();
	}

	// Compute one pseudo-random hash value for each corner
	let hash = permute_4(
		permute_4(permute_4(vec4(i0.z, i1.z, i2.z, i3.z)) + vec4(i0.y, i1.y, i2.y, i3.y))
			+ vec4(i0.x, i1.x, i2.x, i3.x),
	);

	// Compute generating gradients from a Fibonacci spiral on the unit sphere
	let theta = hash * 3.883222077; // 2*pi/golden ratio
	let sz = hash * -0.006920415 + 0.996539792; // 1-(hash+0.5)*2/289
	let psi = hash * 0.108705628; // 10*pi/289, chosen to avoid correlation

	let ct = theta.cos();
	let st = theta.sin();
	let sz_prime = (1.0 - sz * sz).sqrt(); // s is a point on a unit fib-sphere

	// Rotate gradients by angle alpha around a pseudo-random ortogonal axis
	let (gx, gy, gz) = if norm_rot != 0.0 {
		let alpha = norm_rot * TAU;
		let sa = alpha.sin(); // psi and alpha in different planes
		let ca = alpha.cos();

		let sp = psi.sin(); // q' from psi on equator
		let cp = psi.cos();

		let px = ct * sz_prime; // px = sx
		let py = st * sz_prime; // py = sy
		let pz = sz;

		let ctp = st * sp - ct * cp; // q = (rotate( cross(s,n), dot(s,n))(q')
		let qx = (ctp * st).lerp_vec(sp, sz);
		let qy = (-ctp * ct).lerp_vec(cp, sz);
		let qz = -(py * cp + px * sp);

		(ca * px + sa * qx, ca * py + sa * qy, ca * pz + sa * qz)
	} else {
		(ct * sz_prime, st * sz_prime, sz)
	};

	// Reorganize for dot products below
	let g0 = vec3(gx.x, gy.x, gz.x);
	let g1 = vec3(gx.y, gy.y, gz.y);
	let g2 = vec3(gx.z, gy.z, gz.z);
	let g3 = vec3(gx.w, gy.w, gz.w);

	// Radial decay with distance from each simplex corner
	let w = vec4(
		0.5 - x0.dot(x0),
		0.5 - x1.dot(x1),
		0.5 - x2.dot(x2),
		0.5 - x3.dot(x3),
	)
	.max(Vec4::ZERO);
	let w2 = w * w;
	let w3 = w2 * w;

	// The value of the linear ramp from each of the corners
	let gdotx = vec4(g0.dot(x0), g1.dot(x1), g2.dot(x2), g3.dot(x3));

	// Multiply by the radial decay and sum up the noise value
	let n = w3.dot(gdotx);

	// Compute the first order partial derivatives
	let dw = -6.0 * w2 * gdotx;
	let dn0 = w3.x * g0 + dw.x * x0;
	let dn1 = w3.y * g1 + dw.y * x1;
	let dn2 = w3.z * g2 + dw.z * x2;
	let dn3 = w3.w * g3 + dw.w * x3;
	let gradient = 39.5 * (dn0 + dn1 + dn2 + dn3);

	// Scale the return value to fit nicely into the range [-1, 1]
	(39.5 * n, gradient)
}

/// 3D tiling simplex noise with fixed gradients and analytical derivatives.
///
/// Wrapper around `tiling_rot_noise_3d` with no rotation.
///
/// # Parameters
///
/// * `pos` - Point (x,y,z) to evaluate
/// * `period` - Desired periods along x,y,z (up to 289). Set to 0.0 or negative to skip wrapping along that dimension.
///
/// # Returns
///
/// A tuple containing:
/// * Noise value in approximate range [-1.0, 1.0]
/// * Gradient vector (x,y,z partial derivatives)
pub fn tiling_noise_3d(pos: Vec3, period: Vec3) -> (f32, Vec3) {
	tiling_rot_noise_3d(pos, period, 0.0)
}

/// 3D simplex noise with rotating gradients and analytical derivatives (no tiling).
///
/// Wrapper around `tiling_rot_noise_3d` with no period wrapping.
///
/// # Parameters
///
/// * `pos` - Point (x,y,z) to evaluate
/// * `norm_rot` - Normalized rotation for swirling gradients, where 1.0 is TAU (2π radians)
///
/// # Returns
///
/// A tuple containing:
/// * Noise value in approximate range [-1.0, 1.0]
/// * Gradient vector (x,y,z partial derivatives)
pub fn rot_noise_3d(pos: Vec3, norm_rot: f32) -> (f32, Vec3) {
	tiling_rot_noise_3d(pos, Vec3::ZERO, norm_rot)
}
