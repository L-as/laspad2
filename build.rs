extern crate curl;

use std::io::Write;
use std::fs::File;

use curl::easy::Easy;

fn main() {
	let out = ::std::env::var("OUT_DIR").unwrap();

	println!("cargo:rerun-if-changed=lua-stdlib");

	// Linking to steam's API
	{
		for lib in [
			"libsteam_api.so",
			"steam_api64.dll",
		].iter() {
			let path = &format!("{}/{}", out, lib);
			if File::open(path).is_ok() {
				println!("Skipped already downloaded {}", lib);
				continue;
			};

			let mut target = File::create(path).unwrap();

			let mut easy = Easy::new();
			easy.url(&format!("https://raw.githubusercontent.com/rlabrecque/Steamworks.NET/master/Plugins/x86_64/{}", lib)).unwrap();
			easy.write_function(move |data| {
				Ok(target.write(data).unwrap())
			}).unwrap();
			easy.perform().unwrap();
			println!("Downloaded {}", lib);
		}

		println!("cargo:rustc-link-search=native={}", out);
		println!("cargo:rustc-env=LD_RUN_PATH=$ORIGIN");
	};
}
