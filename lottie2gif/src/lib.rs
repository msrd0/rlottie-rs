//! Convert lottie animations to GIF files.

#![warn(rust_2018_idioms)]
#![deny(unreachable_pub)]

use gif::{DisposalMethod, Encoder, EncodingError, Frame, Repeat};
use rgb::{alt::BGRA8, RGBA8};
use std::{
	fmt::{self, Display, Formatter},
	io::Write,
	slice
};

pub use rlottie::Animation;
use rlottie::Surface;

#[macro_use]
mod util;

/// Color definition used for the background color of the GIF.
#[derive(Clone, Copy, Debug)]
pub struct Color {
	/// The red component of the color.
	pub r: u8,

	/// The green component of the color.
	pub g: u8,

	/// The blue component of the color.
	pub b: u8,

	/// Set this to true to enable a transparent background. Note that since GIF
	/// does not support semi-transparent pixels, those will be rendered as if the
	/// background color was not transparent.
	pub alpha: bool
}

impl Color {
	pub fn from_hex(hex: u32, alpha: bool) -> Result<Self, InvalidHex> {
		if hex >> 24 != 0 {
			return Err(InvalidHex);
		}
		Ok(Self {
			r: ((hex >> 16) & 0xFF) as u8,
			g: ((hex >> 8) & 0xFF) as u8,
			b: (hex & 0xFF) as u8,
			alpha
		})
	}
}

/// Error type returned if an invalid hex value was passed to [`Color::from_hex`].
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct InvalidHex;

impl Display for InvalidHex {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "invalid hex")
	}
}

impl std::error::Error for InvalidHex {}

/// It is very important that [`RGBA8`] and `[u8; 4]` have exactly the same size.
/// This mod does nothing other than fail to compile if that was not the case.
#[allow(dead_code)]
mod rgba_size {
	use rgb::RGBA8;
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
		AssertSize::<{ mem::size_of::<RGBA8>() }>::new().assert_size_u8_4();
		AssertSize::<{ mem::size_of::<[u8; 4]>() }>::new().assert_size_u8_4();
	};
}

auto_vectorize! {
	pub(crate) fn argb_to_rgba(bg: Color, buffer_argb: &[BGRA8], buffer_rgba: &mut [RGBA8]) {
		let bg_r = bg.r as u32;
		let bg_g = bg.g as u32;
		let bg_b = bg.b as u32;

		buffer_argb
			.iter()
			.map(|color| (color.r as u32, color.g as u32, color.b as u32, color.a))
			.map(|(mut r, mut g, mut b, mut a)| {
				if a == 0 {
					r = 0;
					g = 0;
					b = 0;
				}

				let a_neg = (255 - a) as u32;
				r += (bg_r * a_neg) / 255;
				g += (bg_g * a_neg) / 255;
				b += (bg_b * a_neg) / 255;

				if !bg.alpha || a != 0 {
					a = 255;
				}

				(r, g, b, a)
			})
			.zip(buffer_rgba.iter_mut())
			.for_each(|((r, g, b, a), rgba)| {
				rgba.r = r as u8;
				rgba.g = g as u8;
				rgba.b = b as u8;
				rgba.a = a;
			});
	}
}

/// Convert a lottie animation to a GIF file.
///
/// **This is a lossy operation.**
/// GIF does not support full alpha channel. Even if you enable the alpha flag
/// for background color, the rgb value is required. This is because semi-transparent
/// pixels will be converted to non-transparent pixels, adding onto the background
/// color. Only fully transparent pixels will remain transparent.
pub fn convert<W: Write>(
	mut player: Animation,
	bg: Color,
	out: W
) -> Result<(), EncodingError> {
	let size = player.size();
	let framerate = player.framerate();
	let delay = (100.0 / framerate).round() as u16;
	let buffer_len = size.width * size.height;
	let mut surface = Surface::new(size);
	let mut buffer_rgba = vec![RGBA8::default(); buffer_len];
	let frame_count = player.totalframe();

	let mut gif = Encoder::new(out, size.width as _, size.height as _, &[])?;
	gif.set_repeat(Repeat::Infinite)?;
	for frame in 0 .. frame_count {
		player.render(frame, &mut surface);
		argb_to_rgba(bg, surface.data(), &mut buffer_rgba);

		let mut frame = {
			// Safety: The pointer is valid and aligned since it comes from a vec, and we don't
			// use the vec while the slice exists.
			let buffer_rgba = unsafe {
				slice::from_raw_parts_mut(
					buffer_rgba.as_mut_ptr() as *mut u8,
					buffer_len * 4
				)
			};
			Frame::from_rgba_speed(
				size.width as _,
				size.height as _,
				buffer_rgba,
				10
			)
		};
		frame.delay = delay;
		if bg.alpha {
			frame.dispose = DisposalMethod::Background;
		}
		gif.write_frame(&frame)?;
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn color_from_hex() {
		let color = Color::from_hex(0x010203, true).unwrap();
		assert_eq!(color.r, 1);
		assert_eq!(color.g, 2);
		assert_eq!(color.b, 3);
		assert!(color.alpha);
	}

	#[test]
	fn color_from_invalid_hex() {
		let color = Color::from_hex(0xFF00FF00, true);
		assert!(color.is_err());
	}
}
