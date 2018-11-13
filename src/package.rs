use derive_more::{Display, From};
use erroneous::Error as EError;
use std::{
	fs,
	io::{self, Seek, Write},
	path::Path,
};
use zip::{result::ZipError, write::ZipWriter};

use crate::{compile, config::Branch, Project};

#[derive(Debug, Display, EError, From)]
pub enum Error {
	#[display(fmt = "Could not compile project for packaging")]
	CompileError(#[error(source)] compile::Error),
	#[display(fmt = "Could not create Zip archive")]
	ZipError(#[error(source)] ZipError),
}

struct ZipTarget<W: Write + Seek> {
	writer: ZipWriter<W>,
}

impl<W: Write + Seek> compile::Out for ZipTarget<W> {
	fn file(&mut self, src: &Path, dst: &Path) -> Result<(), io::Error> {
		let options =
			zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

		let dst: String = dst
			.to_string_lossy()
			.chars()
			.map(|c| if c == '\\' { '/' } else { c })
			.collect();

		match self.writer.start_file(dst, options) {
			Ok(()) => (),
			Err(ZipError::Io(e)) => return Err(e),
			_ => unreachable!(),
		};

		self.writer.write_all(&fs::read(src)?)
	}

	fn dir(&mut self, _: &Path) -> Result<(), io::Error> {
		Ok(())
	}
}

pub fn package<T: Write + Seek>(project: &Project, branch: &Branch, out: T) -> Result<T, Error> {
	let mut writer = ZipWriter::new(out);
	let options =
		zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
	writer.start_file(".modinfo", options)?;
	writer
		.write_all(format!("name = \"{}\"", branch.name).as_bytes())
		.map_err(ZipError::Io)?;

	let mut target = ZipTarget { writer };
	compile::compile(project, &mut target)?;

	Ok(target.writer.finish()?)
}
