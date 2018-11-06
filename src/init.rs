use failure::*;
use std::{
	fs::{self, File, OpenOptions},
	io::Write,
	path::Path,
};

use common;

#[derive(Debug, Fail)]
enum InitError {
	#[fail(display = "This is already a laspad project!")]
	AlreadyExists,
}

type Result = ::std::result::Result<(), Error>;

pub fn main() -> Result {
	ensure!(!common::is_laspad_project("."), InitError::AlreadyExists);

	File::create("laspad.toml")?.write_all(include_bytes!("../laspad.toml"))?;

	log!(
		"Example laspad.toml created. Please modify it. (Nothing will work properly if you don't)"
	);

	fs::create_dir_all("src")?;

	if Path::new(".git").exists() {
		OpenOptions::new()
			.create(true)
			.append(true)
			.open(".gitignore")?
			.write_all(b"/compiled\n")?;
	};

	Ok(())
}
