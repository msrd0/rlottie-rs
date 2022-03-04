fn main() {
	#[cfg(target_endian = "big")]
	println!("cargo:warning=rlottie-rs has not been tested on big endian targets. It might be unsound. Use at your own risk.");
}
