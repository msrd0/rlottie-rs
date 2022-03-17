//! Convert lottie animations to GIF files.

#![warn(rust_2018_idioms)]
#![deny(unreachable_pub)]

use gif::{DisposalMethod, Encoder, EncodingError, Frame, Repeat};
use rgb::{alt::BGRA8, RGBA8};
use std::{io::Write, slice};

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

	/// Set this to true to enable a transparent background. Note that since GIF
	/// does not support semi-transparent pixels, those will be rendered as if the
	/// background color was not transparent.
	pub alpha: bool
}

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

macro_rules! auto_vectorize {
	(
		pub(crate) fn $ident:ident($($arg_ident:ident : $arg_ty:ty),*) $(-> $ret:ty)? {
			$($body:tt)*
		}
	) => {
		pub(crate) fn $ident($($arg_ident: $arg_ty),*) $(-> $ret)? {
			#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
			#[target_feature(enable = "avx2")]
			#[target_feature(enable = "bmi1")]
			#[target_feature(enable = "bmi2")]
			#[allow(unused_unsafe)]
			unsafe fn avx2($($arg_ident: $arg_ty),*) $(-> $ret)? {
				$($body)*
			}

			#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
			#[target_feature(enable = "sse4.1")]
			#[allow(unused_unsafe)]
			unsafe fn sse4_1($($arg_ident: $arg_ty),*) $(-> $ret)? {
				$($body)*
			}

			fn fallback($($arg_ident: $arg_ty),*) $(-> $ret)? {
				$($body)*
			}

			#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
			if is_x86_feature_detected!("avx2") {
				return unsafe { avx2($($arg_ident),*) };
			}

			#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
			if is_x86_feature_detected!("sse4.1") {
				return unsafe { sse4_1($($arg_ident),*) };
			}

			fallback($($arg_ident),*)
		}
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
	let buffer_len = size.width() as usize * size.height() as usize;
	let mut buffer_argb = Vec::with_capacity(buffer_len);
	let mut buffer_rgba = vec![RGBA8::default(); buffer_len];
	let frame_count = player.totalframe();

	let mut gif = Encoder::new(out, size.width() as _, size.height() as _, &[])?;
	gif.set_repeat(Repeat::Infinite)?;
	for frame in 0 .. frame_count {
		player.render(frame, &mut buffer_argb, size).unwrap();
		argb_to_rgba(bg, &buffer_argb, &mut buffer_rgba);

		let mut frame = {
			// Safety: The pointer is valid and align since it comes from a vec, and we don't
			// use the vec while the slice exists.
			let buffer_rgba = unsafe {
				slice::from_raw_parts_mut(
					buffer_rgba.as_mut_ptr() as *mut u8,
					buffer_len * 4
				)
			};
			Frame::from_rgba_speed(
				size.width() as _,
				size.height() as _,
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
