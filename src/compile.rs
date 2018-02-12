use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

type Result = io::Result<()>;

fn iterate_dir<F>(root: &Path, path: &Path, f: &mut F) -> Result
	where F: FnMut(&Path, &Path) -> Result
{
	for entry in fs::read_dir(path).expect("Attempted to read non-existent directory!") {
		let entry = &entry.expect("Could not access file").path();
		if entry.file_name().unwrap().to_str().unwrap().chars().next().unwrap() != '.' {
			if entry.is_dir() {
				iterate_dir(root, entry, f)?;
			} else {
				f(entry, entry.strip_prefix(root).unwrap())?;
			};
		} else {
			debug!("Ignored file {:?}", entry);
		};
	};
	Ok(())
}

pub fn iterate_files<F>(path: &Path, f: &mut F) -> Result
	where F: FnMut(&Path, &Path) -> Result
{

	if path.join(".update_timestamp").exists() {
		debug!(".update_timestamp exists in {:?}", path);
		iterate_dir(path, path, f)?;
	} else if path.join("laspad.toml").exists() {
		debug!("laspad.toml exists in {:?}", path);
		let src = &path.join("src");
		iterate_dir(src, src, f)?;
		let dependencies = path.join("dependencies");
		if dependencies.exists() {
			for dependency in fs::read_dir(dependencies)? {
				iterate_files(&dependency?.path(), f)?;
			};
		};
	} else if path.join("mod.settings").exists() {
		debug!("mod.settings exists in {:?}", path);
		use regex::Regex;
		lazy_static! {
			static ref SOURCE_RE: Regex = Regex::new(r#"^\s*source_dir\s*=\s*".*?""#).unwrap();
			static ref OUTPUT_RE: Regex = Regex::new(r#"^\s*output_dir\s*=\s*".*?""#).unwrap();
		}
		let modsettings = &String::from_utf8(File::open(path.join("mod.settings"))?.bytes().map(|b| b.unwrap()).collect()).unwrap();
		let source = path.join(&SOURCE_RE.captures(modsettings).unwrap()[1]);
		let output = path.join(&OUTPUT_RE.captures(modsettings).unwrap()[1]);
		if source.exists() {
			iterate_dir(&source, &source, f)?;
		};
		if output.exists() {
			iterate_dir(&source, &source, f)?;
		};
	} else { // just guess
		debug!("Guessing source directory in {:?}", path);
		let mut found = false;
		for source_dir in [
			"source",
			"output",
			"src",
		].iter() {
			let source_dir = &path.join(source_dir);
			if source_dir.exists() {
				found = true;
				trace!("Found {:?} in {:?}", source_dir, path);
				iterate_dir(source_dir, source_dir, f)?;
			};
		};
		if !found {
			iterate_dir(path, path, f)?;
		};
	};
	Ok(())
}

pub fn main() -> Result {
	let dest = Path::new("compiled");

	if dest.exists() {
		fs::remove_dir_all(dest).expect("Couldn't remove directory 'compiled'");
		fs::create_dir(dest).expect("Couldn't create directory 'compiled'");
	}

	iterate_files(&Path::new("."), &mut |path, rel_path| {
		trace!("{:?} < {:?}", rel_path, path);
		let dest = dest.join(rel_path);
		match dest.parent() {
			Some(parent) => fs::create_dir_all(parent).expect("Couldn't create necessary directories for file"),
			None         => {},
		};
		if let Err(e) = fs::hard_link(path, dest) {
			if e.kind() == io::ErrorKind::AlreadyExists {
				warn!("Multiple mods have file {:?}!", rel_path);
				Ok(())
			} else {
				Err(e)
			}
		} else {
			Ok(())
		}
	})
}
