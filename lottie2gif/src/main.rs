use clap::Parser;
use gif::Frame;
use rlottie::{Animation, Size};
use std::fs::File;

#[derive(Parser)]
struct Args {
	#[clap(name = "lottieFileName")]
	lottie_file_name: String,

	#[clap(name = "resolution", default_value = "200x200")]
	resolution: String,

	#[clap(name = "bgColor", default_value = "ffffffff")]
	bg_color: String
}

fn main() {
	let args = Args::parse();

	let path = args.lottie_file_name;
	let (width, height): (u16, u16) = {
		let mut split = args.resolution.split('x');
		(
			split.next().unwrap().parse().expect("Invalid resolution"),
			split
				.next()
				.expect("Resolution must be of form <width>x<height>")
				.parse()
				.expect("Invalid resolution")
		)
	};
	let bg_color = u32::from_str_radix(&args.bg_color, 16).expect("Invalid bgColor");
	let bg_r = ((bg_color >> 16) & 0xFF) as u8;
	let bg_g = ((bg_color >> 8) & 0xFF) as u8;
	let bg_b = (bg_color & 0xFF) as u8;

	let mut player = Animation::from_file(&path).expect("Failed to open file");
	let buffer_len = width as usize * height as usize;
	let mut buffer32 = Vec::with_capacity(buffer_len);
	let mut buffer8 = unsafe {
		buffer32.set_len(buffer_len);
		Vec::from_raw_parts(
			buffer32.as_mut_ptr() as *mut u8,
			buffer_len * 4,
			buffer_len * 4
		)
	};
	let frame_count = player.totalframe();

	let mut gif = gif::Encoder::new(
		File::create(&format!("{}.gif", path)).expect("Failed to create output file"),
		width,
		height,
		&[]
	)
	.expect("Failed to create gif");
	for frame in 0..frame_count {
		player.render(
			frame,
			&mut buffer32,
			Size {
				width: width as _,
				height: height as _
			},
			width as u64 * 4
		);

		for i in 0..buffer_len {
			let a = buffer8[i * 4 + 3];
			if a != 0 {
				let r = buffer8[i * 4 + 2];
				let g = buffer8[i * 4 + 1];
				let b = buffer8[i * 4];

				if a != 0xFF {
					// un premultiply
					let r2 = (bg_r as f32 * (0xFF - a) as f32 / 255.0) as u8;
					let g2 = (bg_g as f32 * (0xFF - a) as f32 / 255.0) as u8;
					let b2 = (bg_b as f32 * (0xFF - a) as f32 / 255.0) as u8;
					buffer8[i * 4] = r + r2;
					buffer8[i * 4 + 1] = g + g2;
					buffer8[i * 4 + 2] = b + b2;
				} else {
					buffer8[i * 4] = r;
					buffer8[i * 4 + 2] = b;
				}
			} else {
				buffer8[i * 4 + 2] = bg_b;
				buffer8[i * 4 + 1] = bg_g;
				buffer8[i * 4] = bg_r;
			}
		}

		gif.write_frame(&Frame::from_rgba(width, height, &mut buffer8))
			.expect("Failed to write frame");
	}

	// buffer8 points into buffer32 so we need to leak buffer8 to
	// avoid causing a double-free
	buffer8.leak();
}
