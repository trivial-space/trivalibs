pub fn f32_to_u8(channel: f32) -> u8 {
	(255.999 * channel.clamp(0.0, 1.0)) as u8
}

pub fn f64_to_u8(channel: f64) -> u8 {
	(255.999 * channel.clamp(0.0, 1.0)) as u8
}
