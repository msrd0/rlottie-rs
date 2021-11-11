use gif::{DisposalMethod, Encoder, EncodingError, Frame, Repeat};
use rlottie::Argb;
use std::io::Write;

pub use rlottie::Animation;

#[derive(Clone, Copy, Debug)]
pub struct Color {
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub alpha: bool
}

fn argb_to_rgba(bg: Color, buffer_len: usize, buffer_argb: &Vec<Argb>, buffer_rgba: &mut Vec<u8>) {
	#[cfg(debug_assertions)]
	{
		assert_eq!(buffer_argb.len(), buffer_len);
		assert_eq!(buffer_rgba.len(), buffer_len * 4);
	}

	for i in 0..buffer_len {
		let mut color = buffer_argb[i];

		if color.a != 0 {
			let factor = (0xFF - color.a) as f32 / 255.0;
			color.r += (bg.r as f32 * factor) as u8;
			color.g += (bg.g as f32 * factor) as u8;
			color.b += (bg.b as f32 * factor) as u8;
		} else {
			color.r = bg.r;
			color.g = bg.g;
			color.b = bg.b;
		}

		buffer_rgba[i * 4] = color.r;
		buffer_rgba[i * 4 + 1] = color.g;
		buffer_rgba[i * 4 + 2] = color.b;
		buffer_rgba[i * 4 + 3] = match color.a {
			0 if bg.alpha => 0,
			_ => 0xFF
		};
	}
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
	let mut buffer_argb = Vec::with_capacity(buffer_len);
	let mut buffer_rgba = vec![0; buffer_len * 4];
	let frame_count = player.totalframe();

	let mut gif = Encoder::new(out, size.width as _, size.height as _, &[])?;
	gif.set_repeat(Repeat::Infinite)?;
	for frame in 0..frame_count {
		player.render(frame, &mut buffer_argb, size).unwrap();

		let start = std::time::Instant::now();
		argb_to_rgba(bg, buffer_len, &buffer_argb, &mut buffer_rgba);
		let elapsed = start.elapsed();
		println!("elapsed: {:?}", elapsed);

		let mut frame = Frame::from_rgba(size.width as _, size.height as _, &mut buffer_rgba);
		frame.delay = delay;
		if bg.alpha {
			frame.dispose = DisposalMethod::Background;
		}
		gif.write_frame(&frame)?;
	}

	Ok(())
}
