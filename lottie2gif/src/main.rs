use clap::Parser;
use lottie2gif::Color;
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
	let bg = Color {
		r: ((bg_color >> 16) & 0xFF) as u8,
		g: ((bg_color >> 8) & 0xFF) as u8,
		b: (bg_color & 0xFF) as u8,
		alpha: true
	};

	println!("Converting file {} ...", path);
	let player = Animation::from_file(&path).expect("Failed to open file");
	let gif_path = format!("{}.gif", path);
	lottie2gif::convert(
		player,
		bg,
		File::create(&gif_path).expect("Failed to create output file")
	)
	.expect("Conversion failed");
}
