// vec3 rgb2hsl( in vec3 c ){
//     vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
//     vec4 p = mix(vec4(c.bg, K.wz),
//                  vec4(c.gb, K.xy),
//                  step(c.b, c.g));
//     vec4 q = mix(vec4(p.xyw, c.r),
//                  vec4(c.r, p.yzx),
//                  step(p.x, c.r));
//     float d = q.x - min(q.w, q.y);
//     float e = 1.0e-10;
//     return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)),
//                 d / (q.x + e),
//                 q.x);
// }

use crate::{smoothstep::Smoothen, step::Step};
use spirv_std::glam::{vec3, vec4, Vec3};
use spirv_std::num_traits::Float;

pub fn rgb2hsl(c: Vec3) -> Vec3 {
	let k = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
	let p = vec4(c.z, c.y, k.w, k.z).lerp(vec4(c.y, c.z, k.x, k.y), c.z.step(c.y));
	let q = vec4(p.x, p.y, p.w, c.x).lerp(vec4(c.x, p.y, p.z, p.x), p.x.step(c.x));
	let d = q.x - q.w.min(q.y);
	let e = 1.0e-10;
	vec3(
		(q.z + (q.w - q.y) / (6.0 * d + e)).abs(),
		d / (q.x + e),
		q.x,
	)
}

//  Function from Iñigo Quiles
//  https://www.shadertoy.com/view/MsS3Wc

pub fn hsl2rgb(c: Vec3) -> Vec3 {
	let rgb = (((c.x * 6.0 + vec3(0.0, 4.0, 2.0) % 6.0) - 3.0).abs() - 1.0).smoothen();
	c.z * Vec3::ONE.lerp(rgb, c.y)
}
