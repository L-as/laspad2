use std::{path::Path, env};
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
