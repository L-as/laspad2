extern crate curl;

use std::io::Write;
use std::fs::File;
use std::path::Path;

use curl::easy::Easy;

fn main() {
	// Linking to steam's API
	{
		for &lib in [
			"libsteam_api.so",
			"steam_api64.dll",
		].iter() {
			let dst = Path::new("3rdparty").join(lib);

			let mut dst = File::create(dst).unwrap();

			let mut easy = Easy::new();
			easy.url(&format!("https://raw.githubusercontent.com/rlabrecque/Steamworks.NET/master/Plugins/x86_64/{}", lib)).unwrap();
			easy.write_function(move |data| {
				Ok(dst.write(data).unwrap())
			}).unwrap();
			easy.perform().unwrap();
		}

		println!("cargo:rustc-link-search=native=3rdparty");
		println!("cargo:rustc-env=LD_RUN_PATH=$ORIGIN");
	};
}
