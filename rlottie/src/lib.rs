#![warn(rust_2018_idioms)]
#![deny(elided_lifetimes_in_paths, unreachable_pub)]

//! Safe Rust bindings to rlottie.

use rgb::alt::BGRA8;
use rlottie_sys::*;
use std::{
	ffi::CString,
	fmt::{self, Debug},
	os::unix::ffi::OsStrExt,
	path::Path
};

fn path_to_cstr<P>(path: P) -> CString
where
	P: AsRef<Path>
{
	let bytes = path.as_ref().as_os_str().as_bytes().to_vec();
	CString::new(bytes).expect("path must not contain nul")
}

/// The size type used by lottie [`Animation`].
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Size {
	pub width: usize,
	pub height: usize
}

impl Size {
	pub const fn new(width: usize, height: usize) -> Self {
		Self { width, height }
	}
}

/// It is very important that [`BGRA8`] and `u32` have exactly the same size. This
/// mod does nothing other than fail to compile if that was not the case.
#[allow(dead_code)]
mod bgra8_size {
	use rgb::alt::BGRA8;
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
		AssertSize::<{ mem::size_of::<BGRA8>() }>::new().assert_size_u32();
		AssertSize::<{ mem::size_of::<u32>() }>::new().assert_size_u32();
	};
}

/// A surface has a fixed size and contains pixel data for it. You can render frames onto
/// the surface.
pub struct Surface {
	data: Vec<BGRA8>,
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
	pub fn data(&self) -> &[BGRA8] {
		&self.data
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

/// A lottie animation.
pub struct Animation(*mut Lottie_Animation_S);

impl Debug for Animation {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Animation").finish_non_exhaustive()
	}
}

impl Drop for Animation {
	fn drop(&mut self) {
		unsafe {
			lottie_animation_destroy(self.0);
		}
	}
}

impl Animation {
	/// Read a lottie animation from file. This file needs to be in JSON format; if
	/// you want to read telegram's tgs files, you need to decompress them first.
	pub fn from_file<P>(path: P) -> Option<Self>
	where
		P: AsRef<Path>
	{
		let path = path_to_cstr(path);
		let ptr = unsafe { lottie_animation_from_file(path.as_ptr()) };
		(!ptr.is_null()).then(|| Self(ptr))
	}

	/// Read a file from memory. External resources are resolved relative to
	/// `resource_path`.
	pub fn from_data<P>(data: String, key: String, resource_path: P) -> Option<Self>
	where
		P: AsRef<Path>
	{
		let data = CString::new(data).expect("data must not contain nul");
		let key = CString::new(key).expect("key must not contain nul");
		let resource_path = path_to_cstr(resource_path);
		let ptr = unsafe {
			lottie_animation_from_data(
				data.as_ptr(),
				key.as_ptr(),
				resource_path.as_ptr()
			)
		};
		(!ptr.is_null()).then(|| Self(ptr))
	}

	/// Return the default viewport size of this animation.
	pub fn size(&self) -> Size {
		let mut size = Size {
			width: 0,
			height: 0
		};
		unsafe {
			lottie_animation_get_size(self.0, &mut size.width, &mut size.height);
		}
		size
	}

	/// Return the total duration of this animation in seconds.
	pub fn duration(&self) -> f64 {
		unsafe { lottie_animation_get_duration(self.0) }
	}

	/// Return the total number of frames in this animation.
	pub fn totalframe(&self) -> usize {
		unsafe { lottie_animation_get_totalframe(self.0) }
	}

	/// Return the default framerate of this animation.
	pub fn framerate(&self) -> f64 {
		unsafe { lottie_animation_get_framerate(self.0) }
	}

	/// Maps position to frame number and returns it.
	pub fn frame_at_pos(&self, pos: f32) -> usize {
		unsafe { lottie_animation_get_frame_at_pos(self.0, pos) }
	}

	/// Render the contents of a frame onto the surface.
	pub fn render(&mut self, frame_num: usize, surface: &mut Surface) {
		unsafe {
			lottie_animation_render(
				self.0,
				frame_num,
				surface.as_mut_ptr(),
				surface.width(),
				surface.height(),
				surface.width() * 4
			);
			surface.set_len();
		}
	}
}
