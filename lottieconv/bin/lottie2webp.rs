use clap::Parser;
use lottieconv::Converter;
use rlottie::Animation;
use std::{fs::File, io::Write, path::PathBuf};

/// Convert lottie files WEBP.
#[derive(Parser)]
struct Args {
	/// The location of the lottie file.
	#[clap(name = "lottieFileName")]
	lottie_file_name: PathBuf,

	/// The output file.
	#[clap(short = 'o', long = "out", name = "output")]
	output: Option<PathBuf>
}

fn main() {
	let args = Args::parse();
	let path = args.lottie_file_name;

	println!("Converting file {} ...", path.display());
	let player = Animation::from_file(&path).expect("Failed to open file");
	let webp_path = match args.output {
		Some(path) => path,
		None => {
			let file_stem = path.file_stem().expect("Missing file stem");
			let mut webp_path = path.clone();
			let mut webp_file_name = file_stem.to_owned();
			webp_file_name.push(".webp");
			webp_path.set_file_name(webp_file_name);
			webp_path
		}
	};
	let webp = Converter::new(player)
		.webp()
		.and_then(Converter::convert)
		.expect("Conversion failed");
	let mut out = File::create(&webp_path).expect("Failed to create output file");
	out.write_all(&webp).expect("Failed to write output file");
}
