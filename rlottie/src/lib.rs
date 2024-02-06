#![allow(clippy::tabs_in_doc_comments)]
#![warn(rust_2018_idioms)]
#![deny(elided_lifetimes_in_paths, unreachable_pub)]

//! Safe Rust bindings to rlottie.
//!
//! # Example
//!
//! ```rust,edition2021,no_run
//! use rlottie::{Animation, Surface};
//!
//! # fn first(path_to_lottie_json: std::path::PathBuf) -> Option<()> {
//! let mut animation = Animation::from_file(path_to_lottie_json)?;
//! # Some(())
//! # }
//! # fn second(mut animation: Animation) {
//! let size = animation.size();
//! let mut surface = Surface::new(size);
//! for frame in 0 .. animation.totalframe() {
//! 	animation.render(frame, &mut surface);
//! 	for (x, y, color) in surface.pixels() {
//! 		println!("frame {frame} at ({x}, {y}): {color:?}");
//! 	}
//! }
//! # }
//! ```

use rlottie_sys::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
	ffi::CString,
	fmt::{self, Debug},
	mem,
	os::unix::ffi::OsStrExt,
	path::Path,
	ptr::NonNull,
	slice
};

#[cfg(target_endian = "little")]
pub type Color<T = u8> = rgb::alt::BGRA<T>;
#[cfg(target_endian = "big")]
pub type Color<T = u8> = rgb::alt::ARGB<T>;

pub type Rgb<T = f64> = rgb::RGB<T>;

#[doc(hidden)]
#[deprecated = "This type is not guaranteed to have Bgra layout. Use Color instead"]
pub type Bgra<T = u8> = Color<T>;

fn color_is_valid(Rgb { r, g, b }: Rgb) -> bool {
	(0.0 ..= 1.0).contains(&r)
		&& (0.0 ..= 1.0).contains(&g)
		&& (0.0 ..= 1.0).contains(&b)
}

fn path_to_cstr<P>(path: P) -> CString
where
	P: AsRef<Path>
{
	let bytes = path.as_ref().as_os_str().as_bytes().to_vec();
	CString::new(bytes).expect("path must not contain nul")
}

/// The size type used by lottie [`Animation`].
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Size {
	pub width: usize,
	pub height: usize
}

impl Size {
	pub const fn new(width: usize, height: usize) -> Self {
		Self { width, height }
	}
}

/// It is very important that [`Color`] and `u32` have exactly the same size. This
/// mod does nothing other than fail to compile if that was not the case.
#[allow(dead_code)]
mod color_size {
	use super::Color;
	use std::{marker::PhantomData, mem};

	#[derive(Default)]
	struct AssertSize<const N: usize>(PhantomData<[(); N]>);

	impl<const N: usize> AssertSize<N> {
		const fn new() -> Self {
			Self(PhantomData)
		}
	}

	impl AssertSize<4> {
		const fn assert_size_u32(self) {}
	}

	const _: () = {
		AssertSize::<{ mem::size_of::<Color>() }>::new().assert_size_u32();
		AssertSize::<{ mem::size_of::<u32>() }>::new().assert_size_u32();
	};
}

/// A surface has a fixed size and contains pixel data for it. You can render frames onto
/// the surface.
pub struct Surface {
	data: Vec<Color>,
	size: Size
}

impl Surface {
	/// Create a new surface with a fixed size.
	pub fn new(size: Size) -> Self {
		Self {
			data: Vec::with_capacity(size.width * size.height),
			size
		}
	}

	/// Return the size of the surface.
	pub fn size(&self) -> Size {
		self.size
	}

	/// Return the width of the surface.
	pub fn width(&self) -> usize {
		self.size.width
	}

	/// Return the height of the surface.
	pub fn height(&self) -> usize {
		self.size.height
	}

	/// Return the pixel data of the surface.
	pub fn data(&self) -> &[Color] {
		&self.data
	}

	/// Return the pixel data of the surface. You should prefer [`data()`] unless you
	/// absolutely need owned access to the data.
	pub fn into_data(self) -> Vec<Color> {
		self.data
	}

	/// Return the raw pixel data of the surface.
	pub fn data_as_bytes(&self) -> &[u8] {
		// Safety: We are not mutating the surface data for the lifetime of the returned
		// slice, and the memory was properly allocated by Vec, so this is fine.
		unsafe {
			slice::from_raw_parts(
				self.data.as_ptr() as *const u8,
				self.data.len() * mem::size_of::<Color>()
			)
		}
	}

	/// Returns an iterator over the pixels of the surface.
	pub fn pixels(&self) -> impl Iterator<Item = (usize, usize, Color)> + '_ {
		let width = self.width();
		self.data().iter().enumerate().map(move |(i, color)| {
			let x = i % width;
			let y = i / width;
			(x, y, *color)
		})
	}

	/// Return a pointer to the pixel data.
	fn as_mut_ptr(&mut self) -> *mut u32 {
		self.data.as_mut_ptr() as *mut u32
	}

	/// Set the length of the pixel data to `width * height`.
	unsafe fn set_len(&mut self) {
		self.data.set_len(self.width() * self.height())
	}
}

impl AsRef<[u8]> for Surface {
	fn as_ref(&self) -> &[u8] {
		self.data_as_bytes()
	}
}

/// A lottie animation.
pub struct Animation(NonNull<Lottie_Animation_S>);

impl Debug for Animation {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Animation").finish_non_exhaustive()
	}
}

impl Drop for Animation {
	fn drop(&mut self) {
		unsafe {
			lottie_animation_destroy(self.0.as_ptr());
		}
	}
}

impl Animation {
	fn from_ptr(ptr: *mut Lottie_Animation_S) -> Option<Self> {
		(!ptr.is_null()).then(|| {
			// Safety: This is only called if ptr is non null
			Self(unsafe { NonNull::new_unchecked(ptr) })
		})
	}

	/// Constructs an animation object from file path. This file needs to be in JSON
	/// format; if you want to read telegram's tgs files, you need to decompress
	/// them first.
	///
	/// Note that the rlottie library might cache the file and/or its resources.
	pub fn from_file<P>(path: P) -> Option<Self>
	where
		P: AsRef<Path>
	{
		let path = path_to_cstr(path);
		let ptr = unsafe { lottie_animation_from_file(path.as_ptr()) };
		Self::from_ptr(ptr)
	}

	/// Constructs an animation object from JSON string data. External resources are
	/// resolved relative to `resource_path`.
	///
	/// Note that the `cache_key` might be used by the rlottie library to cache the
	/// json data and/or its resources.
	///
	/// This method will panic if json_data or cache_key contain nul bytes.
	pub fn from_data<D, K, P>(
		json_data: D,
		cache_key: K,
		resource_path: P
	) -> Option<Self>
	where
		D: Into<Vec<u8>>,
		K: Into<Vec<u8>>,
		P: AsRef<Path>
	{
		let json_data =
			CString::new(json_data).expect("json_data must not contain nul");
		let cache_key = CString::new(cache_key).expect("key must not contain nul");
		let resource_path = path_to_cstr(resource_path);
		let ptr = unsafe {
			lottie_animation_from_data(
				json_data.as_ptr(),
				cache_key.as_ptr(),
				resource_path.as_ptr()
			)
		};
		Self::from_ptr(ptr)
	}

	/// Return the default viewport size of this animation.
	pub fn size(&self) -> Size {
		let mut size = Size {
			width: 0,
			height: 0
		};
		unsafe {
			lottie_animation_get_size(
				self.0.as_ptr(),
				&mut size.width,
				&mut size.height
			);
		}
		size
	}

	/// Return the total duration of this animation in seconds.
	pub fn duration(&self) -> f64 {
		unsafe { lottie_animation_get_duration(self.0.as_ptr()) }
	}

	/// Return the total number of frames in this animation.
	pub fn totalframe(&self) -> usize {
		unsafe { lottie_animation_get_totalframe(self.0.as_ptr()) }
	}

	/// Return the default framerate of this animation.
	pub fn framerate(&self) -> f64 {
		unsafe { lottie_animation_get_framerate(self.0.as_ptr()) }
	}

	/// Maps position to frame number and returns it.
	pub fn frame_at_pos(&self, pos: f32) -> usize {
		unsafe { lottie_animation_get_frame_at_pos(self.0.as_ptr(), pos) }
	}

	/// Render the contents of a frame onto the surface.
	pub fn render(&mut self, frame_num: usize, surface: &mut Surface) {
		unsafe {
			lottie_animation_render(
				self.0.as_ptr(),
				frame_num,
				surface.as_mut_ptr(),
				surface.width(),
				surface.height(),
				surface.width() * 4
			);
			surface.set_len();
		}
	}

	pub fn set_fill_color(&mut self, keypath: &str, color: Rgb) {
		assert!(color_is_valid(color), "color is not valid");
		let keypath = CString::new(keypath).unwrap();
		let Rgb { r, g, b } = color;
		unsafe {
			lottie_animation_property_override(
				self.0.as_ptr(),
				rlottie_sys::Lottie_Animation_Property::LOTTIE_ANIMATION_PROPERTY_FILLCOLOR,
				keypath.as_ptr(),
				r,
				g,
				b,
			);
		}
	}

	pub fn set_fill_opacity(&mut self, keypath: &str, opacity: f64) {
		assert!(
			(0.0 ..= 100.0).contains(&opacity),
			"opacity values must be between 0.0 and 100.0"
		);
		let keypath = CString::new(keypath).unwrap();
		unsafe {
			lottie_animation_property_override(
				self.0.as_ptr(),
				rlottie_sys::Lottie_Animation_Property::LOTTIE_ANIMATION_PROPERTY_FILLOPACITY,
				keypath.as_ptr(),
				opacity,
			);
		}
	}

	pub fn set_stroke_color(&mut self, keypath: &str, color: Rgb) {
		assert!(color_is_valid(color), "color is not valid");
		let keypath = CString::new(keypath).unwrap();
		let Rgb { r, g, b } = color;
		unsafe {
			lottie_animation_property_override(
				self.0.as_ptr(),
				rlottie_sys::Lottie_Animation_Property::LOTTIE_ANIMATION_PROPERTY_STROKECOLOR,
				keypath.as_ptr(),
				r,
				g,
				b,
			);
		}
	}

	pub fn set_stroke_opacity(&mut self, keypath: &str, opacity: f64) {
		assert!(
			(0.0 ..= 100.0).contains(&opacity),
			"opacity values must be between 0.0 and 100.0"
		);
		let keypath = CString::new(keypath).unwrap();
		unsafe {
			lottie_animation_property_override(
				self.0.as_ptr(),
				rlottie_sys::Lottie_Animation_Property::LOTTIE_ANIMATION_PROPERTY_STROKEOPACITY,
				keypath.as_ptr(),
				opacity,
			);
		}
	}

	pub fn set_stroke_width(&mut self, keypath: &str, width: f64) {
		let keypath = CString::new(keypath).unwrap();
		unsafe {
			lottie_animation_property_override(
				self.0.as_ptr(),
				rlottie_sys::Lottie_Animation_Property::LOTTIE_ANIMATION_PROPERTY_STROKEWIDTH,
				keypath.as_ptr(),
				width,
			);
		}
	}

	pub fn set_tr_position(&mut self, keypath: &str, x: f64, y: f64) {
		let keypath = CString::new(keypath).unwrap();
		unsafe {
			lottie_animation_property_override(
				self.0.as_ptr(),
				rlottie_sys::Lottie_Animation_Property::LOTTIE_ANIMATION_PROPERTY_TR_POSITION,
				keypath.as_ptr(),
				x,
				y,
			);
		}
	}

	pub fn set_tr_scale(&mut self, keypath: &str, width: f64, height: f64) {
		let keypath = CString::new(keypath).unwrap();
		unsafe {
			lottie_animation_property_override(
				self.0.as_ptr(),
				rlottie_sys::Lottie_Animation_Property::LOTTIE_ANIMATION_PROPERTY_TR_SCALE,
				keypath.as_ptr(),
				width,
				height,
			);
		}
	}

	pub fn set_tr_rotation(&mut self, keypath: &str, rotation: f64) {
		let keypath = CString::new(keypath).unwrap();
		unsafe {
			lottie_animation_property_override(
				self.0.as_ptr(),
				rlottie_sys::Lottie_Animation_Property::LOTTIE_ANIMATION_PROPERTY_TR_ROTATION,
				keypath.as_ptr(),
				rotation,
			);
		}
	}
}
