use derive_more::Display;
use erroneous::Error as EError;
use std::{
	fs,
	io,
	path::{Path, PathBuf},
};

#[derive(Debug, Display, EError)]
#[display(fmt = "Could not read `{}`", "path.display()")]
pub struct ReadError {
	#[allow(unused_variables)]
	pub path: PathBuf,
	#[allow(unused_variables)]
	#[error(source)]
	pub source: io::Error,
}

pub fn read_to_string(path: impl AsRef<Path>) -> Result<String, ReadError> {
	let path = path.as_ref();
	fs::read_to_string(path).map_err(|e| ReadError {
		path:   path.to_owned(),
		source: e,
	})
}
