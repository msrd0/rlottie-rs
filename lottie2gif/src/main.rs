use clap::Parser;
use lottie2gif::Color;
use rlottie::Animation;
use std::{fs::File, path::PathBuf};

/// Convert lottie files GIF.
#[derive(Parser)]
struct Args {
	/// The location of the lottie file.
	#[clap(name = "lottieFileName")]
	lottie_file_name: PathBuf,

	/// The background color in hexadecimal format.
	#[clap(name = "bgColor", default_value = "0")]
	bg_color: String,

	/// Disable background transparency.
	#[clap(long = "non-transparent")]
	bg_non_transparent: bool,

	/// The output file.
	#[clap(short = 'o', long = "out", name = "output")]
	output: Option<PathBuf>
}

fn main() {
	let args = Args::parse();

	let path = args.lottie_file_name;
	let file_stem = path.file_stem().expect("Missing file stem");
	let bg_color = u32::from_str_radix(&args.bg_color, 16).expect("Invalid bgColor");
	let bg = Color {
		r: ((bg_color >> 16) & 0xFF) as u8,
		g: ((bg_color >> 8) & 0xFF) as u8,
		b: (bg_color & 0xFF) as u8,
		alpha: !args.bg_non_transparent
	};

	println!("Converting file {} ...", path.display());
	let player = Animation::from_file(&path).expect("Failed to open file");
	let gif_path = match args.output {
		Some(path) => path,
		None => {
			let mut gif_path = path.clone();
			let mut gif_file_name = file_stem.to_owned();
			gif_file_name.push(".gif");
			gif_path.set_file_name(gif_file_name);
			gif_path
		}
	};
	lottie2gif::convert(
		player,
		bg,
		File::create(&gif_path).expect("Failed to create output file")
	)
	.expect("Conversion failed");
}
