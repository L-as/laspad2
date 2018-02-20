extern crate curl;

use std::{env, fs::File, io::Write, path::Path};
use curl::easy::Easy;

const JQUERY_URL: &'static str = "https://code.jquery.com/jquery-3.3.1.min.js";

fn main() {
	let out_dir = env::var("OUT_DIR").unwrap();

	for &(url, path) in [
		(JQUERY_URL, "jquery.js"),
	].iter() {
		let mut file = File::create(Path::new(&out_dir).join(path)).unwrap();
		let mut easy = Easy::new();
		easy.url(url).unwrap();
		{
			let mut transfer = easy.transfer();
			transfer.write_function(|data| {
				file.write_all(data).unwrap();
				Ok(data.len())
			}).unwrap();
			transfer.perform().unwrap();
		};
	}

	println!("cargo:rustc-link-search=native=3rdparty");
	println!("cargo:rustc-env=LD_RUN_PATH=$ORIGIN");
}
