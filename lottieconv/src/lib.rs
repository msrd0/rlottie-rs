#![allow(clippy::tabs_in_doc_comments)]
#![warn(rust_2018_idioms)]
#![deny(elided_lifetimes_in_paths, unreachable_pub)]

//! Convert lottie animations to GIF or WEBP format.
//!
//! ## Examples
//!
//! This example shows how to use this crate to convert a lottie animation to a gif
//! animation. This requires the `gif` feature which is enabled by default.
//!
//! ```rust,edition2018,no_run
//! use lottieconv::{Animation, Converter, Rgba};
//! use std::fs::File;
//!
//! # fn first() -> Option<()> {
//! let animation = Animation::from_file("animation.json")?;
//! # None
//! # }
//! # fn second() -> Result<(), std::io::Error> {
//! let mut out = File::create("animation.gif")?;
//! # Ok(())
//! # }
//! # fn third(animation: Animation, mut out: File) -> Result<(), gif_crate::EncodingError> {
//! Converter::new(animation)
//! 	.gif(Rgba::new_alpha(0, 0, 0, true), &mut out)?
//! 	.convert()?;
//! # Ok(())
//! # }
//! ```
//!
//! This example shows how to use this crate to convert a lottie animation to a webp
//! animation. This requires the `webp` feature which is enabled by default.
//!
//! ```rust,edition2018,no_run
//! use lottieconv::{Animation, Converter};
//! use std::{fs::File, io::Write as _};
//!
//! # fn first() -> Option<()> {
//! let animation = Animation::from_file("animation.json")?;
//! # None
//! # }
//! # fn second(animation: Animation) -> Result<(), webp_animation::Error> {
//! let webp_data = Converter::new(animation).webp()?.convert()?;
//! # Ok(())
//! # }
//! # fn third(webp_data: webp_animation::WebPData) -> Result<(), std::io::Error> {
//! let mut out = File::create("animation.webp")?;
//! out.write_all(&webp_data)?;
//! # Ok(())
//! # }
//! ```

use rgb::RGBA8;
use rlottie::Surface;
use std::slice;

#[doc(no_inline)]
pub use rlottie::{Animation, Size};

#[cfg(any(feature = "gif", feature = "webp"))]
#[macro_use]
mod util;

mod convert;
use convert::{Convert, DummyConvert};

#[cfg(feature = "gif")]
mod gif;
#[cfg(feature = "gif")]
use gif::Convert2Gif;

#[cfg(feature = "webp")]
mod webp;
#[cfg(feature = "webp")]
use webp::Convert2Webp;

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

/// This type is used to perform the conversion. It does nothing unless you
/// call [`.convert()`](Self::convert).
#[must_use = "A Converter does nothing unless you call .convert()"]
pub struct Converter<C: Convert> {
	player: Animation,
	size: Size,
	convert: C
}

/// This type is used to build a [`Converter`]. It is created using
/// [`Converter::new()`].
pub struct Builder {
	player: Animation,
	size: Size
}

impl Converter<DummyConvert> {
	/// Return a new converter builder.
	#[allow(clippy::new_ret_no_self)]
	#[must_use]
	pub fn new(player: Animation) -> Builder {
		Builder {
			size: player.size(),
			player
		}
	}
}

impl Builder {
	/// Change the size of the output image.
	#[must_use]
	pub fn with_size(mut self, size: Size) -> Self {
		self.size = size;
		self
	}
}

#[cfg(feature = "gif")]
/// This type is used to define the background of a GIF.
pub type Rgba = rgb::RGBA<u8, bool>;

#[cfg(feature = "gif")]
impl Builder {
	/// Create a converter for lottie animation to a GIF file.
	///
	/// **This is a lossy operation.**
	/// GIF does not support full alpha channel. Even if you enable the alpha flag
	/// for background color, the rgb value is required. This is because semi-transparent
	/// pixels will be converted to non-transparent pixels, adding onto the background
	/// color. Only fully transparent pixels will remain transparent.
	pub fn gif<W: std::io::Write>(
		self,
		bg: Rgba,
		out: W
	) -> gif::Result<Converter<Convert2Gif<W>>> {
		let framerate = self.player.framerate();
		Ok(Converter {
			player: self.player,
			size: self.size,
			convert: Convert2Gif::new(bg, out, self.size, framerate)?
		})
	}
}

#[cfg(feature = "webp")]
impl Builder {
	/// Create a converter for lottie animation to a WEBP file.
	pub fn webp(self) -> webp::Result<Converter<Convert2Webp>> {
		let framerate = self.player.framerate();
		Ok(Converter {
			player: self.player,
			size: self.size,
			convert: Convert2Webp::new(self.size, framerate)?
		})
	}
}

impl<C: Convert> Converter<C> {
	/// Convert lottie animation to the requested format.
	///
	/// The return type of this function depends on the output format. For webp, it
	/// returns the image data ([`WebPData`](webp_animation::WebPData)). For gif, it
	/// returns `()` as the data is written directly to the writer that was passed
	/// to the `.gif(..)` function.
	pub fn convert(mut self) -> Result<C::Out, C::Err> {
		let buffer_len = self.size.width * self.size.height;
		let mut surface = Surface::new(self.size);
		let mut buffer = vec![RGBA8::default(); buffer_len];
		let frame_count = self.player.totalframe();

		for frame in 0 .. frame_count {
			self.player.render(frame, &mut surface);
			self.convert.convert_frame(surface.data(), &mut buffer);

			// Safety: The pointer is valid and aligned since it comes from a vec,
			// and we don't use the vec while the slice exists.
			let data = unsafe {
				slice::from_raw_parts_mut(
					buffer.as_mut_ptr() as *mut u8,
					buffer_len * 4
				)
			};
			self.convert.add_frame(data)?;
		}

		self.convert.finish()
	}
}
