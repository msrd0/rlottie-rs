//! Safe Rust bindings to rlottie.

use rgb::alt::BGRA8;
use rlottie_sys::*;
use std::{ffi::CString, os::unix::ffi::OsStrExt, path::Path, ptr};

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
	width: size_t,
	height: size_t
}

impl Size {
	pub fn width(&self) -> u64 {
		self.width.into()
	}

	pub fn height(&self) -> u64 {
		self.height.into()
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

/// A lottie animation.
pub struct Animation(*mut Lottie_Animation_S);

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
		(ptr != ptr::null_mut()).then(|| Self(ptr))
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
		(ptr != ptr::null_mut()).then(|| Self(ptr))
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
	pub fn totalframe(&self) -> u64 {
		unsafe { lottie_animation_get_totalframe(self.0) }.into()
	}

	/// Return the default framerate of this animation.
	pub fn framerate(&self) -> f64 {
		unsafe { lottie_animation_get_framerate(self.0) }
	}

	/// Maps position to frame number and returns it.
	pub fn frame_at_pos(&self, pos: f32) -> u64 {
		unsafe { lottie_animation_get_frame_at_pos(self.0, pos) }.into()
	}

	/// Render the contents of a frame into the buffer at a certain viewport size.
	///
	/// The buffer's capacity must be at least `size.width * size.height`. It's
	/// initial length or content doesn't matter. The first `size.width * size.height`
	/// bytes of the buffer will be written to; and it's length will be set exactly
	/// to `size.width * size.height`.
	///
	/// This operation will fail only if the buffer's capacity isn't large enough.
	pub fn render(
		&mut self,
		frame_num: u64,
		buffer: &mut Vec<BGRA8>,
		size: Size
	) -> Result<(), RenderError> {
		let buffer_len = (size.width * size.height) as usize;
		if buffer.capacity() < buffer_len {
			return Err(RenderError);
		}
		unsafe {
			lottie_animation_render(
				self.0,
				frame_num.try_into().unwrap(),
				buffer.as_mut_ptr() as *mut u32,
				size.width,
				size.height,
				size.width * 4
			);
			buffer.set_len(buffer_len);
		}
		Ok(())
	}
}

#[derive(Debug)]
pub struct RenderError;
