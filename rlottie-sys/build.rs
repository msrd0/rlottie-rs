use std::{env, path::PathBuf};

fn main() {
	pkg_config::Config::new()
		.probe("rlottie")
		.expect("Unable to find rlottie");

	println!("cargo:rerun-if-changed=wrapper.h");
	let bindings = bindgen::Builder::default()
		.formatter(bindgen::Formatter::Prettyplease)
		.header("wrapper.h")
		.parse_callbacks(Box::new(bindgen::CargoCallbacks))
		.newtype_enum(".*")
		.size_t_is_usize(true)
		.use_core()
		.generate()
		.expect("Unable to generate bindings");

	let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
	bindings
		.write_to_file(out_path.join("bindings.rs"))
		.expect("Couldn't write bindings!");
}
