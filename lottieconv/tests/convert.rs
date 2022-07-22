use lottieconv::{Animation, Converter, Rgba, Size};
use std::{fs::File, io::Write as _, path::Path, process::Command};

fn xdg_open(path: &Path) {
	Command::new("xdg-open")
		.arg(format!("file://{}", path.canonicalize().unwrap().display()))
		.status()
		.unwrap();
}

#[test]
#[ignore = "requires manual inspection"]
fn example2gif() {
	let animation = Animation::from_file("tests/example.json")
		.expect("Failed to load animation");
	let path: &Path = "tests/example.gif".as_ref();
	let mut out = File::create(path).expect("Failed to create file");
	Converter::new(animation)
		.with_size(Size::new(512, 512))
		.gif(Rgba::new_alpha(0xFF, 0xFF, 0xFF, true), &mut out)
		.expect("Failed to create converter")
		.convert()
		.expect("Failed to convert file");
	xdg_open(path);
}

#[test]
#[ignore = "requires manual inspection"]
fn example2webp() {
	let animation = Animation::from_file("tests/example.json")
		.expect("Failed to load animation");
	let webp_data = Converter::new(animation)
		.with_size(Size::new(512, 512))
		.webp()
		.expect("Failed to create converter")
		.convert()
		.expect("Failed to convert file");
	let path: &Path = "tests/example.webp".as_ref();
	let mut out = File::create(path).expect("Failed to create file");
	out.write_all(&webp_data).expect("Failed to write file");
	xdg_open(path);
}
