use std::{env, fs::remove_dir_all, path::PathBuf, process::Command};

fn main() {
	let lottie_version = "v0.2";
	let output = PathBuf::from(env::var("OUT_DIR").unwrap());
	let clone_dest_dir = format!("lottie-source-{}", lottie_version);
	let lottie_source_dir = output.join(&clone_dest_dir);
	let lottie_install_dir= output.join(format!("lottie-install-{}", lottie_version));
	let _ = remove_dir_all(&lottie_source_dir); //avoid error at cloning if folder already exist
	let status = Command::new("git")
		.current_dir(&output)
		.arg("clone")
		.arg("--depth=1")
		.arg("--branch")
		.arg(lottie_version)
		.arg("https://github.com/Samsung/rlottie.git")
		.arg(&clone_dest_dir)
		.status()
		.unwrap();
	if !status.success() {
		panic!("fetch lottie failed");
	}
	// rlottie needs patch for newer compilers
	let status = Command::new("wget").current_dir(&lottie_source_dir).args(["-qO","patch","https://github.com/Samsung/rlottie/commit/2d7b1fa2b005bba3d4b45e8ebfa632060e8a157a.patch"])
    .status().unwrap();
	if !status.success() {
		panic!("fetch patch failed");
	}
	let status = Command::new("patch")
		.current_dir(&lottie_source_dir)
		.args(["-N", "-p1", "-i", "patch"])
		.status()
		.unwrap();
	if !status.success() {
		panic!("apply patch failed");
	}
	// build lottie
	let dst = cmake::Config::new(lottie_source_dir)
		.define("CMAKE_INSTALL_PREFIX", &lottie_install_dir)
		.build();
	
	panic!("sucess");
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
