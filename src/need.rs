use failure::*;
use std::{
	fs::{self, File, OpenOptions},
	io::Write,
	path::{Path, PathBuf},
};

use common;

#[derive(Debug, Fail)]
pub enum NeedError {
	#[fail(display = "Dependency already exists")]
	AlreadyExists,
	#[fail(display = "Mod ID is not valid")]
	InvalidModId,
}

type Result = ::std::result::Result<(), Error>;

pub fn main(dep: &str) -> Result {
	common::find_project()?;

	ensure!(
		u64::from_str_radix(dep, 16).is_ok(),
		NeedError::InvalidModId
	);

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
			.write_all(
				format!(
					"/dependencies/{}/*\n!/dependencies/{}/.laspad_dummy\n",
					dep, dep
				)
				.as_bytes(),
			)?;
	};

	log!("Added {} as dependency; NB: Contents are not downloaded automatically: you must update first!", dep);
	Ok(())
}
