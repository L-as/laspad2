use failure::*;
use lazy_static::lazy_static;
use std::{
	fs::{self, File},
	io::Read,
	path::Path,
};
use walkdir::WalkDir;

use crate::common;

type Result = ::std::result::Result<(), Error>;

fn iterate_dir<F>(root: &Path, f: &mut F) -> Result
where
	F: FnMut(&Path, &Path) -> Result,
{
	for entry in WalkDir::new(root).into_iter().filter_entry(|e| {
		e.file_name()
			.to_str()
			.map_or(true, |s| s.chars().next() != Some('.'))
	}) {
		let entry = entry?;
		let rel = entry.path().strip_prefix(root)?;
		f(root, rel)?;
	}

	Ok(())
}

fn iterate_files<F>(path: &Path, f: &mut F) -> Result
where
	F: FnMut(&Path, &Path) -> Result,
{
	if path.join(".update_timestamp").exists() {
		debug!(".update_timestamp exists in {}", path.display());
		iterate_dir(path, f)?;
	} else if common::is_laspad_project(path) {
		debug!("laspad project exists in {}", path.display());
		let dependencies = &path.join("dependencies");
		if dependencies.exists() {
			for dependency in fs::read_dir(dependencies)? {
				iterate_files(&dependency?.path(), f)?;
			}
		};
		let src = &path.join("src");
		if src.exists() {
			iterate_dir(src, f)?;
		} else {
			warn!("Found no source directory in {}", path.display());
		};
	} else if path.join("mod.settings").exists() {
		debug!("mod.settings exists in {}", path.display());
		use regex::Regex;
		lazy_static! {
			static ref SOURCE_RE: Regex = Regex::new(r#"source_dir\s*=\s*"(.*?)""#).unwrap();
			static ref OUTPUT_RE: Regex = Regex::new(r#"output_dir\s*=\s*"(.*?)""#).unwrap();
		}
		let modsettings = &String::from_utf8(
			File::open(path.join("mod.settings"))?
				.bytes()
				.map(|b| b.unwrap())
				.collect(),
		)?;
		let mut found = false;
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
			warn!("Found no source directory in {}", path.display());
		};
	} else {
		// just guess
		debug!("Guessing source directory in {}", path.display());
		let mut found = false;
		for source_dir in ["source", "output", "src"].iter() {
			let source_dir = &path.join(source_dir);
			if source_dir.exists() {
				found = true;
				trace!("Found {} in {}", source_dir.display(), path.display());
				iterate_dir(source_dir, f)?;
			};
		}
		if !found {
			iterate_dir(path, f)?;
		};
	};
	Ok(())
}

pub fn main() -> Result {
	common::find_project()?;

	let dst = Path::new("compiled");

	iterate_files(&Path::new("."), &mut |root, path| {
		let dst = &dst.join(path);
		let src = root.join(path);
		if src.is_dir() {
			trace!("DIRECTORY {}", path.display());
			fs::create_dir_all(dst)
				.with_context(|_| format!("Could not create directory {}", path.display()))?;
		} else {
			trace!("{}: {}", root.display(), path.display());
			if dst.exists() {
				fs::remove_file(dst)?
			};
			fs::hard_link(src, dst)?;
		};
		Ok(())
	})
}
