use std::{
	fs::{self, File},
	io::Write,
	path::Path,
};

use curl::easy::Easy;

fn main() {
	// Linking to steam's API
	{
		fs::create_dir_all("3rdparty").unwrap();

		for &lib in ["libsteam_api.so", "steam_api64.dll"].iter() {
			let dst = Path::new("3rdparty").join(lib);
			if dst.exists() {
				continue;
			};

			let mut dst = File::create(dst).unwrap();

			let mut easy = Easy::new();
			easy.url(&format!("https://raw.githubusercontent.com/rlabrecque/Steamworks.NET/master/Plugins/x86_64/{}", lib)).unwrap();
			easy.write_function(move |data| Ok(dst.write(data).unwrap()))
				.unwrap();
			easy.perform().unwrap();
		}

		println!("cargo:rustc-link-search=native=3rdparty");
		println!("cargo:rustc-env=LD_RUN_PATH=$ORIGIN");
	};
}
