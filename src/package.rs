use failure::Fallible;
use std::{
	fs::{self, File},
	io::{Seek, Write},
};
use walkdir::WalkDir;
use zip::write::ZipWriter;

use crate::{common, compile, config};

pub fn zip<T: Write + Seek>(branch_name: &str, out: T) -> Fallible<T> {
	let config = config::get()?;
	let branch = config.get(branch_name, crate::steam::Item(0))?.unwrap();

	log!(1; "Zipping up files");

	let mut out = ZipWriter::new(out);

	let options =
		zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

	out.start_file(".modinfo", options)?;
	out.write_all(format!("name = \"{}\"", branch.name()?).as_bytes())
		.expect("Could not write to zip archive!");

	compile::main()?;
	for entry in WalkDir::new("compiled").follow_links(true) {
		let entry = entry?;
		let entry = entry.path();
		if entry.is_file() {
			let rel = entry.strip_prefix("compiled")?;
			log!(2; "{} < {}", rel.display(), entry.display());
			out.start_file(
				rel.to_str()
					.unwrap()
					.clone()
					.chars()
					.map(|c| if cfg!(windows) && c == '\\' { '/' } else { c })
					.collect::<String>(),
				options,
			)?;
			out.write_all(&fs::read(entry)?)
				.expect("Could not write to zip archive!");
		}
	}

	Ok(out.finish()?)
}

pub fn main(branch: &str, path: &str) -> Fallible<()> {
	common::find_project()?;
	zip(branch, File::create(path)?)?;
	Ok(())
}
