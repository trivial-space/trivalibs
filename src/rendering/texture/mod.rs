pub fn render_rgba<F: Fn(u32, u32) -> [f32; 4]>(width: u32, height: u32, f: F) -> Vec<[f32; 4]> {
    let mut pixels = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            pixels.push(f(x, y));
        }
    }
    pixels
}

pub fn render_rgba_double<F: Fn(u32, u32) -> [f64; 4]>(
    width: u32,
    height: u32,
    f: F,
) -> Vec<[f64; 4]> {
    let mut pixels = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            pixels.push(f(x, y));
        }
    }
    pixels
}

fn clamp(x: f32, min: f32, max: f32) -> f32 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

pub fn rgba_f32_to_u8(pixels: Vec<[f32; 4]>) -> Vec<[u8; 4]> {
    pixels
        .into_iter()
        .map(|[r, g, b, a]| {
            [
                (255.999 * clamp(r, 0.0, 1.0)) as u8,
                (255.999 * clamp(g, 0.0, 1.0)) as u8,
                (255.999 * clamp(b, 0.0, 1.0)) as u8,
                (255.999 * clamp(a, 0.0, 1.0)) as u8,
            ]
        })
        .collect()
}

pub fn rgba_f64_to_u8(pixels: Vec<[f64; 4]>) -> Vec<[u8; 4]> {
    pixels
        .into_iter()
        .map(|[r, g, b, a]| {
            [
                (255.999 * r) as u8,
                (255.999 * g) as u8,
                (255.999 * b) as u8,
                (255.999 * a) as u8,
            ]
        })
        .collect()
}

pub fn rgba_u8_to_buffer(pixels: Vec<[u8; 4]>) -> Vec<u8> {
    pixels
        .into_iter()
        .flat_map(|[r, g, b, a]| vec![r, g, b, a])
        .collect()
}
