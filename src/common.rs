use std::{
	path::{Path, PathBuf},
	env
};
use failure::*;

pub fn find_project() -> Result<(), Error> {
	while !Path::new("laspad.toml").exists() {
		if let Some(parent) = env::current_dir()?.parent() {
			env::set_current_dir(&parent)?;
		} else {
			bail!("This is not a laspad project!");
		};
	};
	Ok(())
}

#[cfg(windows)]
pub fn get_ns2() -> PathBuf {
	PathBuf::from("C:/Program Files (x86)/Steam/steamapps/common/Natural Selection 2/x64")
}

#[cfg(not(windows))]
pub fn get_ns2() -> PathBuf {
	Path::new(&env::var_os("HOME").unwrap()).join(".local/share/Steam/steamapps/common/Natural Selection 2/x64")
}
