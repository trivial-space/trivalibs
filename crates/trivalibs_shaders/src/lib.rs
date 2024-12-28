use spirv_std::{
	glam::{Vec2, Vec4},
	Image, Sampler,
};

/// Performs a single direction of the separable Gaussian blur kernel.
///
/// # Arguments
///
/// * `image` - The image to be blurred.
/// * `sampler` - The sampler used for sampling the image.
/// * `diameter` - The diameter (not the radius) of the circle of confusion for this fragment.
/// * `uv` - The texture coordinates of the fragment.
/// * `res` - The resolution of the image.
/// * `dir` - The vector, in screen-space units, from one sample to the next. For a horizontal blur this will be `vec2(1.0, 0.0)`; for a vertical blur this will be `vec2(0.0, 1.0)`.
///
/// # Returns
///
/// The resulting color of the fragment.
///
pub fn gaussian_blur(
	image: &Image!(2D, type=f32, sampled),
	sampler: &Sampler,
	diameter: f32,
	uv: Vec2,
	res: Vec2,
	dir: Vec2,
) -> Vec4 {
	// Usually σ (the standard deviation) is half the radius, and the radius is
	// half the CoC. So we multiply by 0.25.
	let sigma = diameter * 0.25;

	// 1.5σ is a good, somewhat aggressive default for support—the number of
	// texels on each side of the center that we process.
	let support = (sigma * 1.5).ceil() as i32;
	let offset = dir / res;

	// The probability density function of the Gaussian blur is (up to constant factors) `exp(-1 / 2σ² *
	// x²). We precalculate the constant factor here to avoid having to
	// calculate it in the inner loop.
	let exp_factor = -1.0 / (2.0 * sigma * sigma);

	// Accumulate samples on both sides of the current texel. Go two at a time,
	// taking advantage of bilinear filtering.
	let mut sum = image.sample(*sampler, uv);
	let mut weight_sum = 1.0;

	for i in (1..=support).step_by(2) {
		// This is a well-known trick to reduce the number of needed texture
		// samples by a factor of two. We seek to accumulate two adjacent
		// samples c₀ and c₁ with weights w₀ and w₁ respectively, with a single
		// texture sample at a carefully chosen location. Observe that:
		//
		//     k ⋅ lerp(c₀, c₁, t) = w₀⋅c₀ + w₁⋅c₁
		//
		//                              w₁
		//     if k = w₀ + w₁ and t = ───────
		//                            w₀ + w₁
		//
		// Therefore, if we sample at a distance of t = w₁ / (w₀ + w₁) texels in
		// between the two texel centers and scale by k = w₀ + w₁ afterward, we
		// effectively evaluate w₀⋅c₀ + w₁⋅c₁ with a single texture lookup.
		let j = i as f32;
		let w0 = (exp_factor * j * j).exp();
		let w1 = (exp_factor * (j + 1.0) * (j + 1.0)).exp();
		let uv_offset = offset * (j + w1 / (w0 + w1));
		let weight = w0 + w1;

		sum += (image.sample(*sampler, uv + uv_offset) + image.sample(*sampler, uv - uv_offset))
			* weight;
		weight_sum += weight * 2.0;
	}

	return sum / weight_sum;
}

/// Performs a box blur in a single direction of the separable box blur kernel.
///
/// # Arguments
///
/// * `image` - The image to be blurred.
/// * `sampler` - The sampler used for sampling the image.
/// * `diameter` - The diameter (not the radius) of the circle of confusion for this fragment.
/// * `uv` - The texture coordinates of the fragment.
/// * `res` - The resolution of the image.
/// * `dir` - The vector, in screen-space units, from one sample to the next. This need not be horizontal or vertical.
///
/// # Returns
///
/// The resulting color of the fragment.
pub fn box_blur(
	image: &Image!(2D, type=f32, sampled),
	sampler: &Sampler,
	diameter: f32,
	uv: Vec2,
	res: Vec2,
	dir: Vec2,
) -> Vec4 {
	let support = (diameter * 0.5).ceil() as i32;
	let offset = dir / res;

	// Accumulate samples in a single direction.
	let mut sum = image.sample(*sampler, uv);
	for i in 1..=support {
		sum += image.sample(*sampler, uv + offset * (i as f32))
			+ image.sample(*sampler, uv - offset * (i as f32));
	}

	return sum / (1.0 + (support as f32) * 2.0);
}
