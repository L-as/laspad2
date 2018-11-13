use derive_more::{Display, From};
use erroneous::Error as EError;
use git2::{Oid, Repository};
use joinery::Joinable;
use serde_derive::Deserialize;
use std::{
	fs::{self, File, OpenOptions},
	io::{self, Write},
	path::{Path, PathBuf},
};

use crate::{
	config::{self, Config},
	download,
	item::Item,
	util,
};

pub struct Project {
	pub config: Config,
	pub path:   PathBuf,
}

#[derive(Debug, Display, EError)]
pub enum NewError {
	#[display(fmt = "Could not create example config")]
	ConfigCreation(#[error(source)] io::Error),
	#[display(fmt = "Could not create source code directory `src`")]
	SrcCreation(#[error(source)] io::Error),
	#[display(fmt = "Could not modify .gitignore")]
	IgnoreModification(#[error(source)] io::Error),
}

#[derive(Debug, Display, EError, From)]
pub enum GetError {
	#[display(fmt = "{}", _0)]
	ReadError(#[error(defer)] util::ReadError),
	#[display(fmt = "Could not read configuration")]
	FromStr(#[error(source)] config::GetError),
}

#[derive(Debug, Display, EError, From)]
pub enum UpdateError {
	#[display(fmt = "Could not update mod {}", _0)]
	DownloadError(Item, #[error(source)] download::Error),
	#[display(fmt = "The workshop item {} is not a dependency of this project!", _0)]
	NotFound(Item),
}

#[derive(Clone, From)]
pub struct Dependency {
	pub item: Option<Item>,
	pub path: PathBuf,
}

impl Dependency {
	pub fn name(&self) -> Result<String, util::ReadError> {
		#[derive(Deserialize)]
		struct ModInfo {
			name: String,
		}

		let name = match util::read_to_string(self.path.join(".modinfo")) {
			Ok(modinfo) => {
				let modinfo: Result<ModInfo, _> = toml::from_str(&modinfo);
				modinfo.map(|m| m.name).ok()
			},
			Err(ref e) if e.source.kind() == io::ErrorKind::NotFound => None,
			e => return e,
		};

		Ok(name.unwrap_or_else(|| self.path.file_name().expect("No filename for dependency").to_string_lossy().into()))
	}

	pub fn url(&self) -> Option<String> {
		if let Some(item) = self.item {
			Some(item.url())
		} else {
			let repo = Repository::open(&self.path).ok();
			let origin = repo.as_ref().and_then(|r| r.find_remote("origin").ok());
			let url = origin.as_ref().and_then(|o| o.url());
			url.map(|s| s.into())
		}
	}
}

impl Project {
	pub const DEPENDENCIES_PATH: &'static str = "dependencies";
	pub const DEPENDENCIES_STEAM_PATH: &'static str = ".dependencies_steam";
	pub const SOURCE_PATH: &'static str = "src";

	pub fn src(&self) -> PathBuf {
		self.path.join(Project::SOURCE_PATH)
	}

	pub fn get(path: impl AsRef<Path>) -> Result<Option<Self>, GetError> {
		let path = path.as_ref();
		let config = Config::get(path)?;
		let path = path.into();
		Ok(config.map(|config| Project { config, path }))
	}

	pub fn new(path: impl AsRef<Path>) -> Result<Self, NewError> {
		let path = path.as_ref();

		File::create(&path.join("laspad.toml"))
			.and_then(|mut f| Config::EXAMPLE.map_or(Ok(()), |e| f.write_all(e.as_ref())))
			.map_err(NewError::ConfigCreation)?;

		println!(
			"Example config in laspad.toml created. Please modify it. (Nothing will work properly if you don't)",
		);

		fs::create_dir_all(path.join(Project::SOURCE_PATH)).map_err(NewError::SrcCreation)?;

		let gitignore = ["compiled", Project::DEPENDENCIES_STEAM_PATH]
			.iter()
			.map(|s| format!("/{}\n", s))
			.join_concat()
			.to_string();

		if path.join(".git").exists() {
			OpenOptions::new()
				.create(true)
				.append(true)
				.open(".gitignore")
				.and_then(|mut f| f.write_all(gitignore.as_bytes()))
				.map_err(NewError::IgnoreModification)?;
		};

		Ok(Self::get(path).expect("Failed to get project after creating it").expect("Failed to properly create project"))
	}

	pub fn path_for_item(&self, i: Item) -> PathBuf {
		let mut path = self.path.join(Project::DEPENDENCIES_STEAM_PATH);
		path.push(format!("{:X}", i));
		path
	}

	pub fn update(&self, i: Vec<Item>) -> Result<(), UpdateError> {
		use rayon::prelude::*;
		use self::UpdateError::*;

		i.into_par_iter().try_for_each(|item| {
			if !self.config.deps.contains(&item) {
				return Err(NotFound(item));
			}

			download::download(item, self.path_for_item(item))
				.map_err(|e| DownloadError(item, e))
		})?;
		Ok(())
	}

	pub fn dependencies(&self) -> Result<Vec<Dependency>, io::Error> {
		fn dependencies_steam<'a>(this: &'a Project) -> impl Iterator<Item = Dependency> + 'a {
			this.config.deps.iter().map(move |&item| Dependency {
				item: Some(item),
				path: this.path_for_item(item),
			})
		}

		let deps = self.path.join(Project::DEPENDENCIES_PATH);
		if deps.exists() {
			fs::read_dir(deps)?
				.map(|d| {
					Ok(Dependency {
						item: None,
						path: d?.path(),
					})
				})
				.chain(dependencies_steam(self).map(Ok))
				.collect()
		} else {
			dependencies_steam(self).map(Ok).collect()
		}
	}

	pub fn head(&self) -> Option<Oid> {
		let repo = Repository::open(&self.path).ok();
		let head = repo.as_ref().and_then(|r| r.head().ok());
		let commit = head.and_then(|h| h.peel_to_commit().ok());
		commit.map(|c| c.id())
	}
}
