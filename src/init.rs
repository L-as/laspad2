use std::process::exit;
use std::fs::{self, File, OpenOptions};
use std::io::{Result, Write};

pub fn main(git: bool) -> Result<()> {
	if File::open("laspad.toml").is_ok() {
		eprintln!("This is already a laspad project!");
		exit(1);
	};

	File::create("laspad.toml")?.write_all(include_bytes!("../laspad.toml"))?;

	println!("Example laspad.toml created. Please modify it.");

	fs::create_dir_all("src")?;

	if git {
		OpenOptions::new()
			.create(true)
			.append(true)
			.open(".gitignore")?
			.write_all(b"/compiled\n")?;
	};

	Ok(())
}
