use rlottie_sys::*;
use std::{ffi::CString, os::unix::ffi::OsStrExt, path::Path};

fn path_to_cstr<P>(path: P) -> CString
where
	P: AsRef<Path>
{
	let bytes = path.as_ref().as_os_str().as_bytes().to_vec();
	CString::new(bytes).expect("path must not contain nul")
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Size {
	width: u64,
	height: u64
}

pub struct LottieAnimation(*mut Lottie_Animation_S);

impl Drop for LottieAnimation {
	fn drop(&mut self) {
		unsafe {
			lottie_animation_destroy(self.0);
		}
	}
}

impl LottieAnimation {
	pub fn from_file<P>(path: P) -> Self
	where
		P: AsRef<Path>
	{
		let path = path_to_cstr(path);
		Self(unsafe { lottie_animation_from_file(path.as_ptr()) })
	}

	pub fn from_data<P>(data: String, key: String, resource_path: P) -> Self
	where
		P: AsRef<Path>
	{
		let data = CString::new(data).expect("data must not contain nul");
		let key = CString::new(key).expect("key must not contain nul");
		let resource_path = path_to_cstr(resource_path);
		Self(unsafe {
			lottie_animation_from_data(data.as_ptr(), key.as_ptr(), resource_path.as_ptr())
		})
	}

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

	pub fn duration(&self) -> f64 {
		unsafe { lottie_animation_get_duration(self.0) }
	}

	pub fn totalframe(&self) -> u64 {
		unsafe { lottie_animation_get_totalframe(self.0) }
	}

	pub fn framerate(&self) -> f64 {
		unsafe { lottie_animation_get_framerate(self.0) }
	}

	pub fn render_tree(&mut self, frame_num: u64, size: Size) -> LayerNode {
		unsafe {
			LayerNode(lottie_animation_render_tree(
				self.0,
				frame_num,
				size.width,
				size.height
			))
		}
	}

	pub fn frame_at_pos(&self, pos: f32) -> u64 {
		unsafe { lottie_animation_get_frame_at_pos(self.0, pos) }
	}

	pub fn render(
		&mut self,
		frame_num: u64,
		buffer: &mut Vec<u32>,
		size: Size,
		bytes_per_line: u64
	) {
		unsafe {
			lottie_animation_render(
				self.0,
				frame_num,
				buffer.as_mut_ptr(),
				size.width,
				size.height,
				bytes_per_line
			);
		}
	}

	pub fn render_async(
		&mut self,
		frame_num: u64,
		buffer: &mut Vec<u32>,
		size: Size,
		bytes_per_line: u64
	) {
		unsafe {
			lottie_animation_render_async(
				self.0,
				frame_num,
				buffer.as_mut_ptr(),
				size.width,
				size.height,
				bytes_per_line
			);
		}
	}

	pub fn render_flush(&mut self) {
		unsafe {
			lottie_animation_render_flush(self.0);
		}
	}
}

pub struct LayerNode(*const LOTLayerNode);
