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

fn iterate_dir<F>(root: &Path, path: &Path, f: &mut F) -> Result
	where F: FnMut(&Path, &Path) -> Result
{
	for entry in WalkDir::new(path)
		.into_iter()
		.filter_entry(|e| e.file_name().to_str().map_or(false, |s| s.chars().next() == Some('.')))
	{
		let entry = entry?;
		let entry = entry.path();
		let rel   = entry.strip_prefix(root)?;
		f(entry, rel)?;
	};

	Ok(())
}

fn iterate_files<F>(path: &Path, f: &mut F) -> Result
	where F: FnMut(&Path, &Path) -> Result
{
	if path.join(".update_timestamp").exists() {
		log!(2; ".update_timestamp exists in {}", path.display());
		iterate_dir(path, path, f)?;
	} else if path.join("laspad.toml").exists() {
		log!(2; "laspad.toml exists in {}", path.display());
		let src = &path.join("src");
		if src.exists() {
			iterate_dir(src, src, f)?;
		} else {
			elog!(1; "Found no source directory in {}", path.display());
		};
		let dependencies = &path.join("dependencies");
		if dependencies.exists() {
			for dependency in fs::read_dir(dependencies)? {
				iterate_files(&dependency?.path(), f)?;
			};
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
				iterate_dir(&s, &s, f)?;
			};
			source = Some(s.clone());
		};
		if let Some(captures) = OUTPUT_RE.captures(modsettings) {
			let s = path.join(&captures[1]);
			if s.exists() && (source.is_none() || &s != &source.unwrap()) {
				found = true;
				iterate_dir(&s, &s, f)?;
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
	common::find_project()?;

	let dest = Path::new("compiled");

	// We can copy from this location if the files are still valid.
	// This is useful for e.g. overviews, since if a bundled map doesn't change
	// there will be no reason to regenerate it. Then it can simply be
	// copied over from the old directory. If not, the old one will simply
	// disappear at the end of this scope.
	let old = if dest.exists() {
		let dir = Temp::new();
		fs::rename(dest, &dir)?;
		Some(dir)
	} else {None};
	fs::create_dir(dest)?;

	let mut builder = Builder::new(dest.to_path_buf(), old.map(|o| o.to_path_buf()));

	iterate_files(&Path::new("."), &mut |path, rel_path| {
		log!(2; "{} < {}", rel_path.display(), path.display());
		let dest = &dest.join(rel_path);
		if path.is_dir() {
			log!(2; "---- {} ----", rel_path.display());
			fs::create_dir_all(dest).with_context(|_| {
				format!("Could not create directory {}", rel_path.display())
			})?;
		} else {
			if path.exists() {
				elog!("Multiple mods have file {}!", rel_path.display());
			} else {
				builder.build(path, rel_path)?;
			};
		};
		Ok(())
	})?;

	builder.build_rest()
}
