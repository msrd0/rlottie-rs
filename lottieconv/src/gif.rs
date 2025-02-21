use crate::{Rgba, convert::Convert};
use gif_crate::{DisposalMethod, Encoder, EncodingError, Frame, Repeat};
use rgb::{RGBA8, alt::BGRA8};
use rlottie::Size;
use std::io::Write;

auto_vectorize! {
	pub(crate) fn argb_to_rgba(bg: Rgba, buffer_argb: &[BGRA8], buffer_rgba: &mut [RGBA8]) {
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

				if !bg.a || a != 0 {
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

pub(super) type Result<T> = std::result::Result<T, EncodingError>;

pub struct Convert2Gif<W: Write> {
	bg: Rgba,
	width: u16,
	height: u16,
	encoder: Encoder<W>,
	delay: u16
}

impl<W: Write> Convert2Gif<W> {
	pub(super) fn new(bg: Rgba, out: W, size: Size, framerate: f64) -> Result<Self> {
		let width = size.width as u16;
		let height = size.height as u16;
		let mut encoder = Encoder::new(out, width, height, &[])?;
		encoder.set_repeat(Repeat::Infinite)?;
		Ok(Self {
			bg,
			width,
			height,
			encoder,
			delay: (100.0 / framerate).round() as _
		})
	}
}

impl<W: Write> Convert for Convert2Gif<W> {
	type Out = ();
	type Err = EncodingError;

	fn convert_frame(&self, from: &[BGRA8], to: &mut [RGBA8]) {
		argb_to_rgba(self.bg, from, to)
	}

	fn add_frame(&mut self, data: &mut [u8]) -> Result<()> {
		let mut frame = Frame::from_rgba_speed(self.width, self.height, data, 10);
		frame.delay = self.delay;
		if self.bg.a {
			frame.dispose = DisposalMethod::Background;
		}
		self.encoder.write_frame(&frame)
	}

	fn finish(self) -> Result<Self::Out> {
		Ok(())
	}
}
