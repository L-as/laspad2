use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use failure::*;

use logger::*;

type Result = ::std::result::Result<(), Error>;

fn iterate_dir<F, G>(root: &Path, path: &Path, f: &mut F, g: &mut G, log: &Log) -> Result
	where
	F: FnMut(&Path, &Path) -> Result,
	G: FnMut(&Path)        -> Result
{
	for entry in fs::read_dir(path)? {
		let entry = &entry?.path();
		if entry.file_name().unwrap().to_str().unwrap().chars().next().unwrap() != '.' {
			let rel = entry.strip_prefix(root)?;
			if entry.is_dir() {
				g(rel)?;
				iterate_dir(root, entry, f, g, log)?;
			} else {
				f(entry, rel)?;
			};
		} else {
			log!(log, 2; "Ignored file {}", entry.display());
		};
	};
	Ok(())
}

pub fn iterate_files<F, G>(path: &Path, f: &mut F, g: &mut G, log: &Log) -> Result
	where
	F: FnMut(&Path, &Path) -> Result,
	G: FnMut(&Path)        -> Result
{
	if path.join(".update_timestamp").exists() {
		log!(log, 2; ".update_timestamp exists in {}", path.display());
		iterate_dir(path, path, f, g, log)?;
	} else if path.join("laspad.toml").exists() {
		log!(log, 2; "laspad.toml exists in {}", path.display());
		let src = &path.join("src");
		if src.exists() {
			iterate_dir(src, src, f, g, log)?;
		} else {
			elog!(log, 1; "Found no source directory in {}", path.display());
		};
		let dependencies = &path.join("dependencies");
		if dependencies.exists() {
			for dependency in fs::read_dir(dependencies)? {
				iterate_files(&dependency?.path(), f, g, log)?;
			};
		};
	} else if path.join("mod.settings").exists() {
		log!(log, 2; "mod.settings exists in {}", path.display());
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
				iterate_dir(&s, &s, f, g, log)?;
			};
			source = Some(s.clone());
		};
		if let Some(captures) = OUTPUT_RE.captures(modsettings) {
			let s = path.join(&captures[1]);
			if s.exists() && source.is_none() || &s != &source.unwrap() {
				found = true;
				iterate_dir(&s, &s, f, g, log)?;
			};
		};
		if !found {
			elog!(log; "Found no source directory in {}", path.display());
		};
	} else { // just guess
		log!(log, 2; "Guessing source directory in {}", path.display());
		let mut found = false;
		for source_dir in [
			"source",
			"output",
			"src",
		].iter() {
			let source_dir = &path.join(source_dir);
			if source_dir.exists() {
				found = true;
				log!(log, 2; "Found {} in {}", source_dir.display(), path.display());
				iterate_dir(source_dir, source_dir, f, g, log)?;
			};
		};
		if !found {
			iterate_dir(path, path, f, g, log)?;
		};
	};
	Ok(())
}

pub fn main(log: &Log) -> Result {
	let dest = Path::new("compiled");

	if dest.exists() {
		fs::remove_dir_all(dest)?;
		fs::create_dir(dest)?;
	}

	iterate_files(&Path::new("."), &mut |path, rel_path| {
		log!(log, 2; "{} < {}", rel_path.display(), path.display());
		let dest = dest.join(rel_path);
		if let Err(e) = fs::hard_link(path, dest) {
			if e.kind() == io::ErrorKind::AlreadyExists {
				elog!(log; "Multiple mods have file {}!", rel_path.display());
				Ok(())
			} else {
				bail!(e);
			}
		} else {
			Ok(())
		}
	}, &mut |rel_path| {
		log!(log, 2; "--- {} ---", rel_path.display());
		fs::create_dir_all(dest.join(rel_path)).with_context(|_| {
			format!("Could not create directory {}", rel_path.display())
		})?;
		Ok(())
	}, log)
}
