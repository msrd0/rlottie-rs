use gif::{DisposalMethod, Encoder, EncodingError, Frame, Repeat};
use std::io::Write;

pub use rlottie::Animation;

#[derive(Clone, Copy, Debug)]
pub struct Color {
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub alpha: bool
}

/// Convert a lottie animation to a GIF file.
///
/// **This is a lossy operation.**
/// GIF does not support full alpha channel. Even if you enable the alpha flag
/// for background color, the rgb value is required. This is because semi-transparent
/// pixels will be converted to non-transparent pixels, adding onto the background
/// color. Only fully transparent pixels will remain transparent.
pub fn convert<W: Write>(mut player: Animation, bg: Color, out: W) -> Result<(), EncodingError> {
	let size = player.size();
	let framerate = player.framerate();
	let delay = (100.0 / framerate).round() as u16;
	let buffer_len = size.width as usize * size.height as usize;
	let mut buffer_argb = vec![0; buffer_len];
	let mut buffer_rgba = vec![0; buffer_len * 4];
	let frame_count = player.totalframe();

	let mut gif = Encoder::new(out, size.width as _, size.height as _, &[])?;
	gif.set_repeat(Repeat::Infinite)?;
	for frame in 0..frame_count {
		player.render(frame, &mut buffer_argb, size).unwrap();

		for i in 0..buffer_len {
			let color = buffer_argb[i];

			let a = ((color >> 24) & 0xFF) as u8;
			let mut r = ((color >> 16) & 0xFF) as u8;
			let mut g = ((color >> 8) & 0xFF) as u8;
			let mut b = (color & 0xFF) as u8;
			if a != 0 {
				if a != 0xFF {
					// un premultiply
					r += (bg.r as f32 * (0xFF - a) as f32 / 255.0) as u8;
					g += (bg.g as f32 * (0xFF - a) as f32 / 255.0) as u8;
					b += (bg.b as f32 * (0xFF - a) as f32 / 255.0) as u8;
				}
			} else {
				r = bg.r;
				g = bg.g;
				b = bg.b;
			}

			buffer_rgba[i * 4] = r;
			buffer_rgba[i * 4 + 1] = g;
			buffer_rgba[i * 4 + 2] = b;
			buffer_rgba[i * 4 + 3] = match a {
				0 if bg.alpha => 0,
				_ => 0xFF
			};
		}

		let mut frame = Frame::from_rgba(size.width as _, size.height as _, &mut buffer_rgba);
		frame.delay = delay;
		if bg.alpha {
			frame.dispose = DisposalMethod::Background;
		}
		gif.write_frame(&frame)?;
	}

	Ok(())
}
