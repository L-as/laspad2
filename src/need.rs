use clap::ArgMatches;

use std::fs::{self, File, OpenOptions};
use std::process::exit;
use std::io::{Result, Write};
use std::path::PathBuf;

use update;

pub fn main<'a>(git: bool, matches: &ArgMatches<'a>) -> Result<()> {
	fs::create_dir_all("dependencies")?;

	let dep = matches.value_of("MODID").unwrap().to_uppercase();

	trace!("add {}", dep);

	let path = PathBuf::from(format!("dependencies/{}", dep));
	if path.exists() {
		error!("Dependency already exists!");
		exit(1);
	};

	fs::create_dir(&path)?;
	File::create(path.join(".laspad_dummy"))?;
	update::specific(&dep)?;

	if git {
		OpenOptions::new()
			.create(true)
			.append(true)
			.open(".gitignore")?
			.write_all(format!("/dependencies/{}/*\n!/dependencies/{}/.laspad_dummy\n", dep, dep).as_bytes())?;
	}

	Ok(())
}
