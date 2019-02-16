use derive_more::{Display, From};
use erroneous::Error as EError;
use std::{
	io,
	path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::Project;

#[derive(Debug, Display, EError, From)]
pub enum Error {
	#[display(fmt = "Could not iterate over files in directory")]
	WalkDir(#[error(source)] walkdir::Error),
	#[display(fmt = "Could not create '{}'", "_0.display()")]
	Create(PathBuf, #[error(source)] io::Error),
	#[display(fmt = "Could not read dependencies")]
	Dependencies(#[error(source)] io::Error),
	#[display(fmt = "Could not compile dependency at '{}'", "_0.display()")]
	Dependency(PathBuf, #[error(source)] Box<Error>),
}

pub trait Out {
	fn file(&mut self, src: &Path, dst: &Path) -> Result<(), io::Error>;
	fn dir(&mut self, path: &Path) -> Result<(), io::Error>;
}

fn iterate_dir(root: &Path, out: &mut impl Out) -> Result<(), Error> {
	for entry in WalkDir::new(root).into_iter().filter_entry(|e| {
		e.file_name()
			.to_str()
			.map_or(true, |s| s.chars().next() != Some('.'))
	}) {
		let src = entry?;
		let src = src.path();
		let dst = src
			.strip_prefix(root)
			.expect("Could not strip prefix of path");
		if src.is_dir() {
			out.dir(&dst).map_err(|e| Error::Create(dst.into(), e))?;
		} else {
			out.file(&src, &dst)
				.map_err(|e| Error::Create(dst.into(), e))?;
		}
	}

	Ok(())
}

pub fn compile(project: &Project, out: &mut impl Out) -> Result<(), Error> {
	info!("Compiling project at {}", project.path.display());
	for dep in project.dependencies().map_err(Error::Dependencies)? {
		if let Ok(Some(project)) = Project::get(&dep.path) {
			compile(&project, out).map_err(|e| Error::Dependency(dep.path.into(), e.into()))?;
		} else {
			let src = ["source", "output", "src"]
				.iter()
				.map(|p| dep.path.join(p))
				.find(|p| p.exists())
				.unwrap_or(dep.path.clone());
			info!("Compiling mod in {}", src.display());
			iterate_dir(&src, out)?;
		};
	}
	let src = if let Some((source_dir, output_dir)) = &project.config.source_output_dir {
		let output_dir = project.path.join(output_dir);
		if output_dir.exists() {
			Some(output_dir)
		} else {
			let source_dir = project.path.join(source_dir);
			if source_dir.exists() {
				Some(source_dir)
			} else {
				None
			}
		}
	} else {
		let src = project.src();
		if src.exists() {
			Some(src)
		} else {
			None
		}
	};
	if let Some(src) = src {
		iterate_dir(&src, out)?;
	}
	Ok(())
}
