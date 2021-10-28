use std::{env, path::PathBuf};

fn main() {
	pkg_config::Config::new()
		.atleast_version("0.2")
		.probe("rlottie")
		.expect("Unable to find rlottie");

	println!("cargo:rerun-if-changed=wrapper.h");
	let bindings = bindgen::Builder::default()
		.header("wrapper.h")
		.parse_callbacks(Box::new(bindgen::CargoCallbacks))
		.generate()
		.expect("Unable to generate bindings");

	let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
	bindings
		.write_to_file(out_path.join("bindings.rs"))
		.expect("Couldn't write bindings!");
}
