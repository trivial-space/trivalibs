use trivalibs_nostd::prelude::NumExt;

pub fn f32_to_u8(channel: f32) -> u8 {
	(255.999 * channel.clamp01()) as u8
}

pub fn f64_to_u8(channel: f64) -> u8 {
	(255.999 * channel.clamp01()) as u8
}
