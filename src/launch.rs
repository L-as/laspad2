use std::{
	str::FromStr,
	process::Command,
	path::PathBuf,
	env,
	fmt,
	fs,
};

//use vdf;

use failure::*;

use common::*;

type Result<T> = ::std::result::Result<T, Error>;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Program {
	NS2,
	Editor,
}

impl fmt::Display for Program {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(match *self {
			Program::NS2    => "ns2",
			Program::Editor => "editor",
		})
	}
}

impl FromStr for Program {
	type Err = Error;
	fn from_str(s: &str) -> Result<Self> {
		match s {
			"ns2"    => Ok(Program::NS2),
			"editor" => Ok(Program::Editor),
			_        => Err(format_err!("{} is not a valid Spark program", s)),
		}
	}
}

pub fn main(root: Option<&str>, program: Program) -> Result<()> {
	find_project()?;
	::compile::main()?;

	let path = root.map_or_else(|| get_ns2(), |root| PathBuf::from(root));

	#[cfg(windows)]
	use std::os::windows::fs::symlink_dir as symlink;
	#[cfg(not(windows))]
	use std::os::unix::fs::symlink as symlink;

	let mod_dir = &path.join("../laspad_mod");
	if mod_dir.exists() {fs::remove_file(mod_dir)?};
	let compiled = env::current_dir()?.join("compiled");
	symlink(compiled, mod_dir)?;

	let current_dir = env::current_dir()?;
	env::set_current_dir(path)?;

	let status = match program {
		Program::NS2 => {
			Command::new(if cfg!(windows) {"./NS2.exe"} else {"./ns2_linux"})
				.arg("-game")
				.arg("laspad_mod")
				.status()?
		},
		Program::Editor => {
			ensure!(cfg!(windows), "Editor only works on windows!");
			Command::new("./Editor.exe")
				.arg("-game")
				.arg("laspad_mod")
				.status()?
		},
	};

	env::set_current_dir(current_dir)?;

	ensure!(status.success(), "{} failed: {}", program, status);

	Ok(())
}
