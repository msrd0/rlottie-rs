use crate::convert::Convert;
use rgb::{RGBA8, alt::BGRA8};
use rlottie::Size;
use webp_animation::{Encoder, WebPData};

auto_vectorize! {
	pub(crate) fn bgra_to_rgba(buf_bgra: &[BGRA8], buf_rgba: &mut [RGBA8]) {
		for i in 0..buf_bgra.len() {
			buf_rgba[i].r = buf_bgra[i].r;
			buf_rgba[i].g = buf_bgra[i].g;
			buf_rgba[i].b = buf_bgra[i].b;
			buf_rgba[i].a = buf_bgra[i].a;
		}
	}
}

pub(super) type Result<T> = std::result::Result<T, webp_animation::Error>;

pub struct Convert2Webp {
	encoder: Encoder,
	delay: f64,
	timestamp: f64
}

impl Convert2Webp {
	pub(super) fn new(size: Size, framerate: f64) -> Result<Self> {
		Ok(Self {
			encoder: Encoder::new((size.width as _, size.height as _))?,
			delay: 1000.0 / framerate,
			timestamp: 0.0
		})
	}
}

impl Convert for Convert2Webp {
	type Out = WebPData;
	type Err = webp_animation::Error;

	fn convert_frame(&self, from: &[BGRA8], to: &mut [RGBA8]) {
		bgra_to_rgba(from, to)
	}

	fn add_frame(&mut self, data: &mut [u8]) -> Result<()> {
		self.encoder.add_frame(data, self.timestamp.round() as _)?;
		self.timestamp += self.delay;
		Ok(())
	}

	fn finish(self) -> Result<Self::Out> {
		self.encoder.finalize(self.timestamp.round() as _)
	}
}
