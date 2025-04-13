//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket, Johan Helsing

// Ported from https://github.com/johanhelsing/noisy_bevy from WGSL To Rust-GPU
// Original code at https://github.com/stegu/webgl-noise by Stefan Gustavson

use core::f32::consts::TAU;

use spirv_std::glam::{
	vec2, vec3, vec4, Vec2, Vec2Swizzles, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles,
};

fn step_3(edge: Vec3, x: Vec3) -> Vec3 {
	vec3(
		if edge.x <= x.x { 1.0 } else { 0.0 },
		if edge.y <= x.y { 1.0 } else { 0.0 },
		if edge.z <= x.z { 1.0 } else { 0.0 },
	)
}

fn step_4(edge: Vec4, x: Vec4) -> Vec4 {
	vec4(
		if edge.x <= x.x { 1.0 } else { 0.0 },
		if edge.y <= x.y { 1.0 } else { 0.0 },
		if edge.z <= x.z { 1.0 } else { 0.0 },
		if edge.w <= x.w { 1.0 } else { 0.0 },
	)
}

fn permute_1(x: f32) -> f32 {
	((x * 34.0) + 10.0) * x % 289.0
}

fn permute_3(x: Vec3) -> Vec3 {
	((x * 34.0) + 10.0) * x % Vec3::splat(289.0)
}

fn permute_4(x: Vec4) -> Vec4 {
	((x * 34.0) + 10.0) * x % Vec4::splat(289.0)
}

fn taylor_inv_sqrt_1(r: f32) -> f32 {
	1.79284291400159 - 0.85373472095314 * r
}

fn taylor_inv_sqrt_4(r: Vec4) -> Vec4 {
	1.79284291400159 - 0.85373472095314 * r
}

fn grad_4(j: f32, ip: Vec4) -> Vec4 {
	const ONES: Vec4 = vec4(1.0, 1.0, 1.0, -1.0);

	let tmp = (j * ip.xyz()).fract().floor() * 7.0 * ip.z - 1.0;
	let mut p = vec4(tmp.x, tmp.y, tmp.z, 1.5 - tmp.abs().dot(ONES.xyz()));

	let s = step_4(p, Vec4::ZERO);

	let tmp = (s.xyz() * 2.0 - 1.0) * s.www();
	p.x += tmp.x;
	p.y += tmp.y;
	p.z += tmp.z;

	p
}

// Hashed 2-D gradients with an extra rotation.
// (The constant 0.0243902439 is 1/41)
// vec2 rgrad2(vec2 p, float rot) {
// #if 0
// // Map from a line to a diamond such that a shift maps to a rotation.
//   float u = permute(permute(p.x) + p.y) * 0.0243902439 + rot; // Rotate by shift
//   u = 4.0 * fract(u) - 2.0;
//   // (This vector could be normalized, exactly or approximately.)
//   return vec2(abs(u)-1.0, abs(abs(u+1.0)-2.0)-1.0);
// #else
// // For more isotropic gradients, sin/cos can be used instead.
//   float u = permute(permute(p.x) + p.y) * 0.0243902439 + rot; // Rotate by shift
//   u = fract(u) * 6.28318530718; // 2*pi
//   return vec2(cos(u), sin(u));
// #endif
// }
fn grad_2_r(p: Vec2, rot: f32) -> Vec2 {
	let u = permute_1(permute_1(p.x) + p.y) * 0.024390243902439 + rot; // 1/41, Rotate by shift

	// Map from a line to a diamond such that a shift maps to a rotation.
	// let u = 4.0 * u.fract() - 2.0;
	// (This vector could be normalized, exactly or approximately.)
	// vec2(u.abs() - 1.0, ((u + 1.0).abs() - 2.0).abs() - 1.0)

	// For more isotropic gradients, sin/cos can be used instead.
	let u = TAU * u.fract();
	vec2(u.cos(), u.sin())
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

pub fn simplex_noise_3d(v: Vec3) -> f32 {
	let c = vec2(1.0 / 6.0, 1.0 / 3.0);
	let d = vec4(0.0, 0.5, 1.0, 2.0);

	// first corner
	let i = (v + v.dot(c.yyy())).floor();
	let x0 = v - i + i.dot(c.xxx());

	// other corners
	let g = step_3(x0.yzx(), x0.xyz());
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
	let sh = -step_4(h, Vec4::ZERO);

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

// vec4 grad4(float j, vec4 ip)
//   {
//   const vec4 ones = vec4(1.0, 1.0, 1.0, -1.0);
//   vec4 p,s;

//   p.xyz = floor( fract (vec3(j) * ip.xyz) * 7.0) * ip.z - 1.0;
//   p.w = 1.5 - dot(abs(p.xyz), ones.xyz);
//   s = vec4(lessThan(p, vec4(0.0)));
//   p.xyz = p.xyz + (s.xyz*2.0 - 1.0) * s.www;

//   return p;
//   }

// // (sqrt(5) - 1)/4 = F4, used once below
// #define F4 0.309016994374947451

// float snoise(vec4 v)
//   {
//   const vec4  C = vec4( 0.138196601125011,  // (5 - sqrt(5))/20  G4
//                         0.276393202250021,  // 2 * G4
//                         0.414589803375032,  // 3 * G4
//                        -0.447213595499958); // -1 + 4 * G4

// // First corner
//   vec4 i  = floor(v + dot(v, vec4(F4)) );
//   vec4 x0 = v -   i + dot(i, C.xxxx);

// // Other corners

// // Rank sorting originally contributed by Bill Licea-Kane, AMD (formerly ATI)
//   vec4 i0;
//   vec3 isX = step( x0.yzw, x0.xxx );
//   vec3 isYZ = step( x0.zww, x0.yyz );

//   i0.x = isX.x + isX.y + isX.z;
//   i0.yzw = 1.0 - isX;

//   i0.y += isYZ.x + isYZ.y;
//   i0.zw += 1.0 - isYZ.xy;
//   i0.z += isYZ.z;
//   i0.w += 1.0 - isYZ.z;

//   // i0 now contains the unique values 0,1,2,3 in each channel
//   vec4 i3 = clamp( i0, 0.0, 1.0 );
//   vec4 i2 = clamp( i0-1.0, 0.0, 1.0 );
//   vec4 i1 = clamp( i0-2.0, 0.0, 1.0 );

//   vec4 x1 = x0 - i1 + C.xxxx;
//   vec4 x2 = x0 - i2 + C.yyyy;
//   vec4 x3 = x0 - i3 + C.zzzz;
//   vec4 x4 = x0 + C.wwww;

// // Permutations
//   i = mod289(i);
//   float j0 = permute( permute( permute( permute(i.w) + i.z) + i.y) + i.x);
//   vec4 j1 = permute( permute( permute( permute (
//              i.w + vec4(i1.w, i2.w, i3.w, 1.0 ))
//            + i.z + vec4(i1.z, i2.z, i3.z, 1.0 ))
//            + i.y + vec4(i1.y, i2.y, i3.y, 1.0) ))
//            + i.x + vec4(i1.x, i2.x, i3.x, 1.0 ));

// // Gradients: 7x7x6 points over a cube, mapped onto a 4-cross polytope
// // 7*7*6 = 294, which is close to the ring size 17*17 = 289.
//   vec4 ip = vec4(1.0/294.0, 1.0/49.0, 1.0/7.0, 0.0) ;

//   vec4 p0 = grad4(j0,   ip);
//   vec4 p1 = grad4(j1.x, ip);
//   vec4 p2 = grad4(j1.y, ip);
//   vec4 p3 = grad4(j1.z, ip);
//   vec4 p4 = grad4(j1.w, ip);

// // Normalise gradients
//   vec4 norm = taylorInvSqrt(vec4(dot(p0,p0), dot(p1,p1), dot(p2, p2), dot(p3,p3)));
//   p0 *= norm.x;
//   p1 *= norm.y;
//   p2 *= norm.z;
//   p3 *= norm.w;
//   p4 *= taylorInvSqrt(dot(p4,p4));

// // Mix contributions from the five corners
//   vec3 m0 = max(0.6 - vec3(dot(x0,x0), dot(x1,x1), dot(x2,x2)), 0.0);
//   vec2 m1 = max(0.6 - vec2(dot(x3,x3), dot(x4,x4)            ), 0.0);
//   m0 = m0 * m0;
//   m1 = m1 * m1;
//   return 49.0 * ( dot(m0*m0, vec3( dot( p0, x0 ), dot( p1, x1 ), dot( p2, x2 )))
//                + dot(m1*m1, vec2( dot( p3, x3 ), dot( p4, x4 ) ) ) ) ;

//   }

pub fn simplex_noise_4d(v: Vec4) -> f32 {
	let c = vec4(
		0.138196601125011,  // (5 - sqrt(5))/20  G4
		0.276393202250021,  // 2 * G4
		0.414589803375032,  // 3 * G4
		-0.447213595499958, // -1 + 4 * G4
	);

	// First corner
	let i = (v + v.dot(Vec4::splat(0.309016994374947451))).floor();
	let x0 = v - i + i.dot(c.xxxx());

	// Other corners
	let is_x = step_3(x0.yzw(), x0.xxx());
	let is_yz = step_3(x0.zww(), x0.yyz());

	// Rank sorting originally contributed by Bill Licea-Kane, AMD (formerly ATI)
	let tmp = Vec3::ONE - is_x;
	let mut i0 = vec4(is_x.x + is_x.y + is_x.z, tmp.x, tmp.y, tmp.z);
	i0.y += is_yz.x + is_yz.y;

	let tmp = Vec2::ONE - is_yz.xy();
	i0.z += tmp.x + is_yz.z;
	i0.w += tmp.y + 1.0 - is_yz.z;

	// i0 now contains the unique values 0,1,2,3 in each channel
	let i3 = i0.clamp(Vec4::ZERO, Vec4::ONE);
	let i2 = (i0 - Vec4::ONE).clamp(Vec4::ZERO, Vec4::ONE);
	let i1 = (i0 - Vec4::splat(2.0)).clamp(Vec4::ZERO, Vec4::ONE);

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

pub fn tiling_simplex_noise_2d(uv: Vec2, scale: f32) -> f32 {
	// Map coordinates to circle for seamless wrapping
	let angle_x = uv.x * TAU;
	let angle_y = uv.y * TAU;

	let nx = angle_x.cos() * scale;
	let ny = angle_y.cos() * scale;
	let nz = angle_x.sin() * scale;
	let nw = angle_y.sin() * scale;

	// Get 4D noise value
	simplex_noise_4d(vec4(nx, ny, nz, nw))
}

//
// 2-D tiling simplex noise with rotating gradients,
// but without the analytical derivative.
//

// float psrnoise(vec2 pos, vec2 per, float rot) {
//   // Offset y slightly to hide some rare artifacts
//   pos.y += 0.001;
//   // Skew to hexagonal grid
//   vec2 uv = vec2(pos.x + pos.y*0.5, pos.y);

//   vec2 i0 = floor(uv);
//   vec2 f0 = fract(uv);
//   // Traversal order
//   vec2 i1 = (f0.x > f0.y) ? vec2(1.0, 0.0) : vec2(0.0, 1.0);

//   // Unskewed grid points in (x,y) space
//   vec2 p0 = vec2(i0.x - i0.y * 0.5, i0.y);
//   vec2 p1 = vec2(p0.x + i1.x - i1.y * 0.5, p0.y + i1.y);
//   vec2 p2 = vec2(p0.x + 0.5, p0.y + 1.0);

//   // Integer grid point indices in (u,v) space
//   i1 = i0 + i1;
//   vec2 i2 = i0 + vec2(1.0, 1.0);

//   // Vectors in unskewed (x,y) coordinates from
//   // each of the simplex corners to the evaluation point
//   vec2 d0 = pos - p0;
//   vec2 d1 = pos - p1;
//   vec2 d2 = pos - p2;

//   // Wrap i0, i1 and i2 to the desired period before gradient hashing:
//   // wrap points in (x,y), map to (u,v)
//   vec3 xw = mod(vec3(p0.x, p1.x, p2.x), per.x);
//   vec3 yw = mod(vec3(p0.y, p1.y, p2.y), per.y);
//   vec3 iuw = xw + 0.5 * yw;
//   vec3 ivw = yw;

//   // Create gradients from indices
//   vec2 g0 = rgrad2(vec2(iuw.x, ivw.x), rot);
//   vec2 g1 = rgrad2(vec2(iuw.y, ivw.y), rot);
//   vec2 g2 = rgrad2(vec2(iuw.z, ivw.z), rot);

//   // Gradients dot vectors to corresponding corners
//   // (The derivatives of this are simply the gradients)
//   vec3 w = vec3(dot(g0, d0), dot(g1, d1), dot(g2, d2));

//   // Radial weights from corners
//   // 0.8 is the square of 2/sqrt(5), the distance from
//   // a grid point to the nearest simplex boundary
//   vec3 t = 0.8 - vec3(dot(d0, d0), dot(d1, d1), dot(d2, d2));

//   // Set influence of each surflet to zero outside radius sqrt(0.8)
//   t = max(t, 0.0);

//   // Fourth power of t
//   vec3 t2 = t * t;
//   vec3 t4 = t2 * t2;

//   // Final noise value is:
//   // sum of ((radial weights) times (gradient dot vector from corner))
//   float n = dot(t4, w);

//   // Rescale to cover the range [-1,1] reasonably well
//   return 11.0*n;
// }

pub fn tiling_noise_2d_r(pos: Vec2, period: Vec2, rot: f32) -> f32 {
	// Offset y slightly to hide some rare artifacts
	let pos = vec2(pos.x, pos.y + 0.001);

	// Skew to hexagonal grid
	let uv = vec2(pos.x + pos.y * 0.5, pos.y);

	let i0 = uv.floor();
	let f0 = uv.fract();

	// Traversal order
	let i1 = if f0.x > f0.y {
		vec2(1.0, 0.0)
	} else {
		vec2(0.0, 1.0)
	};

	// Unskewed grid points in (x, y) space
	let p0 = vec2(i0.x - i0.y * 0.5, i0.y);
	let p1 = vec2(p0.x + i1.x - i1.y * 0.5, p0.y + i1.y);
	let p2 = vec2(p0.x + 0.5, p0.y + 1.0);

	// Integer grid point indices in (u, v) space
	// let i1 = i0 + i1;
	// let i2 = i0 + vec2(1.0, 1.0);

	// Vectors in unskewed (x, y) coordinates from each of the simplex corners to the evaluation point
	let d0 = pos - p0;
	let d1 = pos - p1;
	let d2 = pos - p2;

	// Wrap i0, i1, and i2 to the desired period before gradient hashing
	let xw = vec3(p0.x, p1.x, p2.x) % period.x;
	let yw = vec3(p0.y, p1.y, p2.y) % period.y;
	let iuw = xw + 0.5 * yw;
	let ivw = yw;

	// Create gradients from indices
	let g0 = grad_2_r(vec2(iuw.x, ivw.x), rot);
	let g1 = grad_2_r(vec2(iuw.y, ivw.y), rot);
	let g2 = grad_2_r(vec2(iuw.z, ivw.z), rot);

	// Gradients dot vectors to corresponding corners
	// (The derivatives of this are simply the gradients)
	let w = vec3(g0.dot(d0), g1.dot(d1), g2.dot(d2));

	// Radial weights from corners
	// 0.8 is the square of 2/sqrt(5), the distance from
	// a grid point to the nearest simplex boundary
	let t = vec3(0.8 - d0.dot(d0), 0.8 - d1.dot(d1), 0.8 - d2.dot(d2)).max(Vec3::ZERO);

	// Fourth power of t
	let t2 = t * t;
	let t4 = t2 * t2;

	// Final noise value
	// sum of ((radial weights) times (gradient dot vector from corner))
	let n = t4.dot(w);

	// Rescale to cover the range [-1, 1] reasonably well
	11.0 * n
}

//
// 2-D tiling simplex noise with fixed gradients,
// without the analytical derivative.
// This function is implemented as a wrapper to "psrnoise",
// at the minimal cost of three extra additions.
//
pub fn tiling_noise_2d(pos: Vec2, per: Vec2) -> f32 {
	tiling_noise_2d_r(pos, per, 0.0)
}
