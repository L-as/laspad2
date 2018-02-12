use std::fs::{self, File, OpenOptions};
use std::process::exit;
use std::io::{Result, Write};
use std::path::{PathBuf, Path};

use update;

pub fn main(dep: &str) -> Result<()> {
	fs::create_dir_all("dependencies").expect("Could not create 'dependencies' directory");

	let dep = dep.to_uppercase();

	trace!("add {}", dep);

	let path = PathBuf::from(format!("dependencies/{}", dep));
	if path.exists() {
		error!("Dependency already exists!");
		exit(1);
	};

	fs::create_dir(&path).expect("Could not create directory for dependency");
	File::create(path.join(".laspad_dummy")).expect("Could not create .laspad_dummy file");
	update::specific(&dep).expect("Could not update dependency");

	if Path::new(".git").exists() {
		OpenOptions::new()
			.create(true)
			.append(true)
			.open(".gitignore").expect("Could not open/create .gitignore file")
			.write_all(format!("/dependencies/{}/*\n!/dependencies/{}/.laspad_dummy\n", dep, dep).as_bytes())?;
	}

	Ok(())
}
