//! Convert lottie animations to GIF files.

use gif::{DisposalMethod, Encoder, EncodingError, Frame, Repeat};
use rlottie::Argb;
use std::io::Write;

pub use rlottie::Animation;

/// Color definition used for the background color of the GIF.
#[derive(Clone, Copy, Debug)]
pub struct Color {
	/// The red component of the color.
	pub r: u8,

	/// The green component of the color.
	pub g: u8,

	/// The blue component of the color.
	pub b: u8,

	/// Set this to true to enable a transparent background. Note that since GIF does not support
	/// semi-transparent pixels, those will be rendered as if the background color was not
	/// transparent.
	pub alpha: bool
}

#[repr(C)]
struct Rgba {
	r: u8,
	g: u8,
	b: u8,
	a: u8
}

/// It is very important that [`Rgba`] and `[u8; 4]` have exactly the same size. This mod does
/// nothing other than fail to compile if that was not the case.
#[allow(dead_code)]
mod rgba_size {
	use super::Rgba;
	use std::{marker::PhantomData, mem};

	#[derive(Default)]
	struct AssertSize<const N: usize>(PhantomData<[(); N]>);

	impl<const N: usize> AssertSize<N> {
		const fn new() -> Self {
			Self(PhantomData)
		}
	}

	impl AssertSize<4> {
		const fn assert_size_u8_4(self) {}
	}

	const _: () = {
		AssertSize::<{ mem::size_of::<Rgba>() }>::new().assert_size_u8_4();
		AssertSize::<{ mem::size_of::<[u8; 4]>() }>::new().assert_size_u8_4();
	};
}

fn argb_to_rgba(bg: Color, buffer_argb: &Vec<Argb>, buffer_rgba: &mut Vec<u8>) {
	let bg_r = bg.r as f32;
	let bg_g = bg.g as f32;
	let bg_b = bg.b as f32;

	for (i, color) in buffer_argb.iter().enumerate() {
		let idx = i * 4;
		let rgba = &mut buffer_rgba[idx .. idx + 3];
		let rgba: &mut Rgba = unsafe { &mut *(rgba.as_mut_ptr() as *mut Rgba) };

		if color.a != 0 {
			let factor = (0xFF - color.a) as f32 / 255.0;
			rgba.r = color.r + (bg_r * factor) as u8;
			rgba.g = color.g + (bg_g * factor) as u8;
			rgba.b = color.b + (bg_b * factor) as u8;
		} else {
			rgba.r = bg.r;
			rgba.g = bg.g;
			rgba.b = bg.b;
		}
		rgba.a = match color.a {
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
	for frame in 0 .. frame_count {
		player.render(frame, &mut buffer_argb, size).unwrap();
		argb_to_rgba(bg, &buffer_argb, &mut buffer_rgba);

		let mut frame =
			Frame::from_rgba_speed(size.width as _, size.height as _, &mut buffer_rgba, 10);
		frame.delay = delay;
		if bg.alpha {
			frame.dispose = DisposalMethod::Background;
		}
		gif.write_frame(&frame)?;
	}

	Ok(())
}
