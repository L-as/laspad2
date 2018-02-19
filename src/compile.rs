use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::cell::RefCell;
use failure::*;

type Result = ::std::result::Result<(), Error>;

fn iterate_dir<F, G>(root: &Path, path: &Path, f: &mut F, g: &mut G) -> Result
	where
	F: FnMut(&Path, &Path) -> Result,
	G: FnMut(&Path)        -> Result
{
	for entry in fs::read_dir(path).expect("Attempted to read non-existent directory!") {
		let entry = &entry.expect("Could not access file").path();
		if entry.file_name().unwrap().to_str().unwrap().chars().next().unwrap() != '.' {
			let rel = entry.strip_prefix(root)?;
			if entry.is_dir() {
				g(rel)?;
				iterate_dir(root, entry, f, g)?;
			} else {
				f(entry, rel)?;
			};
		} else {
			debug!("Ignored file {:?}", entry);
		};
	};
	Ok(())
}

pub fn iterate_files<F, G>(path: &Path, f: &mut F, g: &mut G, output_err: &RefCell<&mut Write>) -> Result
	where
	F: FnMut(&Path, &Path) -> Result,
	G: FnMut(&Path)        -> Result
{

	if path.join(".update_timestamp").exists() {
		debug!(".update_timestamp exists in {:?}", path);
		iterate_dir(path, path, f, g)?;
	} else if path.join("laspad.toml").exists() {
		debug!("laspad.toml exists in {:?}", path);
		let src = &path.join("src");
		if src.exists() {
			iterate_dir(src, src, f, g)?;
		} else {
			let _ = writeln!(output_err.borrow_mut(), "Found no source directory in {}", path.display());
		};
		let dependencies = &path.join("dependencies");
		if dependencies.exists() {
			for dependency in fs::read_dir(dependencies)? {
				iterate_files(&dependency?.path(), f, g, output_err)?;
			};
		};
	} else if path.join("mod.settings").exists() {
		debug!("mod.settings exists in {:?}", path);
		use regex::Regex;
		lazy_static! {
			static ref SOURCE_RE: Regex = Regex::new(r#"source_dir\s*=\s*"(.*?)""#).unwrap();
			static ref OUTPUT_RE: Regex = Regex::new(r#"output_dir\s*=\s*"(.*?)""#).unwrap();
		}
		let modsettings = &String::from_utf8(File::open(path.join("mod.settings"))?.bytes().map(|b| b.unwrap()).collect())?;
		let mut found  = false;
		let mut source = None;
		if let Some(captures) = SOURCE_RE.captures(modsettings) {
			let s = path.join(&captures[1]);
			if s.exists() {
				found = true;
				iterate_dir(&s, &s, f, g)?;
			};
			source = Some(s.clone());
		};
		if let Some(captures) = OUTPUT_RE.captures(modsettings) {
			let s = path.join(&captures[1]);
			if s.exists() && source.is_none() || &s != &source.unwrap() {
				found = true;
				iterate_dir(&s, &s, f, g)?;
			};
		};
		if !found {
			let _ = writeln!(output_err.borrow_mut(), "Found no source directory in {:?}", path);
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
				iterate_dir(source_dir, source_dir, f, g)?;
			};
		};
		if !found {
			iterate_dir(path, path, f, g)?;
		};
	};
	Ok(())
}

pub fn main(output_err: &mut Write) -> Result {
	let dest = Path::new("compiled");

	if dest.exists() {
		fs::remove_dir_all(dest).expect("Couldn't remove directory 'compiled'");
		fs::create_dir(dest).expect("Couldn't create directory 'compiled'");
	}

	let output_err = RefCell::new(output_err);

	iterate_files(&Path::new("."), &mut |path, rel_path| {
		trace!("{:?} < {:?}", rel_path, path);
		let dest = dest.join(rel_path);
		if let Err(e) = fs::hard_link(path, dest) {
			if e.kind() == io::ErrorKind::AlreadyExists {
				let _ = writeln!(output_err.borrow_mut(), "Multiple mods have file {:?}!", rel_path);
				Ok(())
			} else {
				bail!(e);
			}
		} else {
			Ok(())
		}
	}, &mut |rel_path| {
		trace!("--- {:?} ---", rel_path);
		fs::create_dir_all(dest.join(rel_path)).with_context(|_| {
			format!("Could not create directory {}", rel_path.display())
		})?;
		Ok(())
	}, &output_err)
}
