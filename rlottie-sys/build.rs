use std::{env, path::PathBuf};

fn main() {
	let rlottie_pkg = pkg_config::Config::new()
		.probe("rlottie")
		.expect("Unable to find rlottie");
	
	// Extract include paths from the pkg-config metadata
	let include_args: Vec<String> = rlottie_pkg
		.include_paths
		.iter()
		.map(|path| format!("-I{}", path.display()))
		.collect();
	
	println!("cargo:rerun-if-changed=wrapper.h");
	let bindings = bindgen::Builder::default()
		.formatter(bindgen::Formatter::Prettyplease)
		.header("wrapper.h")
        .clang_args(&include_args) // Add include paths
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
