use clap::Parser;
use gif::Frame;
use rlottie::Animation;
use std::fs::File;

#[derive(Parser)]
struct Args {
	#[clap(name = "lottieFileName")]
	lottie_file_name: String,

	#[clap(name = "bgColor", default_value = "0")]
	bg_color: String
}

fn main() {
	let args = Args::parse();

	let path = args.lottie_file_name;
	let bg_color = u32::from_str_radix(&args.bg_color, 16).expect("Invalid bgColor");
	let bg_r = ((bg_color >> 16) & 0xFF) as u8;
	let bg_g = ((bg_color >> 8) & 0xFF) as u8;
	let bg_b = (bg_color & 0xFF) as u8;

	let mut player = Animation::from_file(&path).expect("Failed to open file");
	let size = player.size();
	let framerate = player.framerate();
	let delay = (1.0 / framerate).round() as u16;
	let buffer_len = size.width as usize * size.height as usize;
	let mut buffer_argb = vec![0; buffer_len];
	let mut buffer_rgb = vec![0; buffer_len * 3];
	let frame_count = player.totalframe();

	let mut gif = gif::Encoder::new(
		File::create(&format!("{}.gif", path)).expect("Failed to create output file"),
		size.width as _,
		size.height as _,
		&[]
	)
	.expect("Failed to create gif");
	for frame in 0..frame_count {
		player.render(frame, &mut buffer_argb, size, size.width * 4);

		for i in 0..buffer_len {
			let color = buffer_argb[i];

			let a = ((color >> 24) & 0xFF) as u8;
			let mut r = ((color >> 16) & 0xFF) as u8;
			let mut g = ((color >> 8) & 0xFF) as u8;
			let mut b = (color & 0xFF) as u8;
			if a != 0 {
				if a != 0xFF {
					// un premultiply
					r += (bg_r as f32 * (0xFF - a) as f32 / 255.0) as u8;
					g += (bg_g as f32 * (0xFF - a) as f32 / 255.0) as u8;
					b += (bg_b as f32 * (0xFF - a) as f32 / 255.0) as u8;
				}
			} else {
				r = bg_r;
				g = bg_g;
				b = bg_b;
			}

			buffer_rgb[i * 3] = r;
			buffer_rgb[i * 3 + 1] = g;
			buffer_rgb[i * 3 + 2] = b;
		}

		let mut frame = Frame::from_rgb(size.width as _, size.height as _, &mut buffer_rgb);
		frame.delay = delay;
		gif.write_frame(&frame).expect("Failed to write frame");
	}
}
