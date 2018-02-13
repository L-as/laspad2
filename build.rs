fn main() {
	println!("cargo:rustc-link-search=native=.");
	println!("cargo:rustc-env=LD_RUN_PATH=$ORIGIN");
}
