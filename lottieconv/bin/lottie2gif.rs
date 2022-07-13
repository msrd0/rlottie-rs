use clap::Parser;
use lottieconv::Converter;
use rgb::RGBA;
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

#[derive(Debug)]
struct InvalidHex;

fn color_from_hex(hex: u32, alpha: bool) -> Result<RGBA<u8, bool>, InvalidHex> {
	if hex >> 24 != 0 {
		return Err(InvalidHex);
	}
	Ok(RGBA {
		r: ((hex >> 16) & 0xFF) as u8,
		g: ((hex >> 8) & 0xFF) as u8,
		b: (hex & 0xFF) as u8,
		a: alpha
	})
}

fn main() {
	let args = Args::parse();

	let path = args.lottie_file_name;
	let file_stem = path.file_stem().expect("Missing file stem");
	let bg_color = u32::from_str_radix(&args.bg_color, 16).expect("Invalid bgColor");
	let bg =
		color_from_hex(bg_color, !args.bg_non_transparent).expect("Invalid bgColor");

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
	Converter::new(player)
		.gif(
			bg,
			File::create(&gif_path).expect("Failed to create output file")
		)
		.and_then(Converter::convert)
		.expect("Conversion failed");
}
