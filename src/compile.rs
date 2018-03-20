use std::{
	fs::{self, File},
	io::Read,
	path::Path,
};
use failure::*;
use walkdir::WalkDir;
use mktemp::Temp;

use common;
use builder::Builder;

type Result = ::std::result::Result<(), Error>;

fn iterate_dir<F>(root: &Path, f: &mut F) -> Result
	where F: FnMut(&Path, &Path) -> Result
{
	for entry in WalkDir::new(root)
		.into_iter()
		.filter_entry(|e| e.file_name().to_str().map_or(true, |s| s.chars().next() != Some('.')))
	{
		let entry = entry?;
		let rel   = entry.path().strip_prefix(root)?;
		f(root, rel)?;
	};

	Ok(())
}

fn iterate_files<F>(path: &Path, f: &mut F) -> Result
	where F: FnMut(&Path, &Path) -> Result
{
	if path.join(".update_timestamp").exists() {
		log!(2; ".update_timestamp exists in {}", path.display());
		iterate_dir(path, f)?;
	} else if common::is_laspad_project(path) {
		log!(2; "laspad project exists in {}", path.display());
		let dependencies = &path.join("dependencies");
		if dependencies.exists() {
			for dependency in fs::read_dir(dependencies)? {
				iterate_files(&dependency?.path(), f)?;
			};
		};
		let src = &path.join("src");
		if src.exists() {
			iterate_dir(src, f)?;
		} else {
			elog!(1; "Found no source directory in {}", path.display());
		};
	} else if path.join("mod.settings").exists() {
		log!(2; "mod.settings exists in {}", path.display());
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
				iterate_dir(&s, f)?;
			};
			source = Some(s.clone());
		};
		if let Some(captures) = OUTPUT_RE.captures(modsettings) {
			let s = path.join(&captures[1]);
			if s.exists() && (source.is_none() || &s != &source.unwrap()) {
				found = true;
				iterate_dir(&s, f)?;
			};
		};
		if !found {
			elog!("Found no source directory in {}", path.display());
		};
	} else { // just guess
		log!(2; "Guessing source directory in {}", path.display());
		let mut found = false;
		for source_dir in [
			"source",
			"output",
			"src",
		].iter() {
			let source_dir = &path.join(source_dir);
			if source_dir.exists() {
				found = true;
				log!(2; "Found {} in {}", source_dir.display(), path.display());
				iterate_dir(source_dir, f)?;
			};
		};
		if !found {
			iterate_dir(path, f)?;
		};
	};
	Ok(())
}

pub fn main() -> Result {
	common::find_project()?;

	let dst = Path::new("compiled");

	// We can copy from this location if the files are still valid.
	// This is useful for e.g. overviews, since if a bundled map doesn't change
	// there will be no reason to regenerate it. Then it can simply be
	// copied over from the old directory. If not, the old one will simply
	// disappear at the end of this scope.
	let old = if dst.exists() {
		let dir = Temp::new_in(".");
		fs::rename(dst, &dir).unwrap();
		Some(dir)
	} else {None};
	fs::create_dir(dst)?;

	let mut builder = Builder::new(dst.to_path_buf(), old.map(|o| o.to_path_buf()));

	iterate_files(&Path::new("."), &mut |root, path| {
		let dst = &dst.join(path);
		let src = root.join(path);
		if src.is_dir() {
			log!(2; "DIRECTORY {}", path.display());
			fs::create_dir_all(dst).with_context(|_| {
				format!("Could not create directory {}", path.display())
			})?;
		} else {
			log!(2; "{}: {}", root.display(), path.display());
			builder.build(&src, path)?;
		};
		Ok(())
	})?;

	builder.build_rest()
}
