use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{PathBuf, Path};
use failure::*;

use logger::*;

#[derive(Debug, Fail)]
pub enum NeedError {
	#[fail(display = "Dependency already exists")]
	AlreadyExists,
}

type Result = ::std::result::Result<(), Error>;

pub fn main(dep: &str, _log: &Log) -> Result {
	fs::create_dir_all("dependencies")?;

	let dep = dep.to_uppercase();

	let path = PathBuf::from(format!("dependencies/{}", dep));
	ensure!(!path.exists(), NeedError::AlreadyExists);

	fs::create_dir(&path)?;
	File::create(path.join(".laspad_dummy"))?;

	if Path::new(".git").exists() {
		OpenOptions::new()
			.create(true)
			.append(true)
			.open(".gitignore")?
			.write_all(format!("/dependencies/{}/*\n!/dependencies/{}/.laspad_dummy\n", dep, dep).as_bytes())?;
	};

	Ok(())
}
