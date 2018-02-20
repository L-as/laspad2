fn main() {
	println!("cargo:rustc-link-search=native=3rdparty");
	println!("cargo:rustc-env=LD_RUN_PATH=$ORIGIN");
}
