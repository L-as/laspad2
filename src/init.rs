use std::fs::{self, File, OpenOptions};
use std::io::{Write};
use std::path::Path;
use failure::*;

#[derive(Debug, Fail)]
enum InitError {
	#[fail(display = "This is already a laspad project!")]
	AlreadyExists,
}

type Result = ::std::result::Result<(), Error>;

pub fn main() -> Result {
	ensure!(!Path::new("laspad.toml").exists(), InitError::AlreadyExists);

	File::create("laspad.toml")?.write_all(include_bytes!("../laspad.toml"))?;

	log!(log; "Example laspad.toml created. Please modify it.");

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
