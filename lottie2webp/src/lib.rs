//! Convert lottie animations to WEBP files.

#![warn(rust_2018_idioms)]
#![deny(elided_lifetimes_in_paths, unreachable_pub)]

use rgb::{alt::BGRA8, RGBA8};
use rlottie::Surface;
use std::slice;
use webp_animation::{Encoder, WebPData};

pub use rlottie::Animation;

#[macro_use]
mod util;

auto_vectorize! {
	pub(crate) fn bgra_to_rgba(buf_bgra: &[BGRA8], buf_rgba: &mut [RGBA8]) {
		for i in 0..buf_bgra.len() {
			buf_rgba[i].r = buf_bgra[i].r;
			buf_rgba[i].g = buf_bgra[i].g;
			buf_rgba[i].b = buf_bgra[i].b;
			buf_rgba[i].a = buf_bgra[i].a;
		}
	}
}

pub fn convert(mut player: Animation) -> Result<WebPData, webp_animation::Error> {
	let size = player.size();
	let framerate = player.framerate();
	let delay = 1000.0 / framerate;
	let buffer_len = size.width * size.height;
	let mut surface = Surface::new(size);
	let mut buffer_rgba = vec![RGBA8::default(); buffer_len];
	let frame_count = player.totalframe();

	let mut webp = Encoder::new((size.width as _, size.height as _))?;
	let mut timestamp: f64 = 0.0;
	for frame in 0 .. frame_count {
		player.render(frame, &mut surface);
		bgra_to_rgba(surface.data(), &mut buffer_rgba);

		{
			// Safety: The pointer is valid and aligned since it comes from a vec, and we don't
			// use the vec while the slice exists.
			let data = unsafe {
				slice::from_raw_parts(
					buffer_rgba.as_ptr() as *const u8,
					buffer_len * 4
				)
			};
			webp.add_frame(data, timestamp.round() as i32)
		}?;
		timestamp += delay;
	}
	webp.finalize(timestamp.round() as i32)
}
