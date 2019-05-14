use derive_more::{Display, From};
use erroneous::Error as EError;
use joinery::Joinable;
use serde_derive::Deserialize;
use std::{
	borrow::Cow,
	collections::HashMap,
	ffi::OsStr,
	fs,
	io,
	path::{Path, PathBuf},
	str::FromStr,
};
use toml;

use crate::{item::Item, project::Project, util};

#[derive(Deserialize)]
pub struct Branch {
	pub name:            String,
	pub tags:            Vec<String>,
	pub autodescription: Option<bool>,
	pub description:     Option<PathBuf>,
	pub description_str: Option<String>,
	pub preview:         Option<PathBuf>,
	pub website:         Option<String>,
	pub item:            Option<Item>,
}

#[derive(Debug, Display, EError, From)]
pub enum DescriptionError {
	#[display(fmt = "{}", _0)]
	ReadError(#[error(defer)] util::ReadError),
	#[display(fmt = "Could not read dependencies")]
	Dependencies(#[error(source)] io::Error),
}

impl Branch {
	pub fn description(&self, project: &Project, item: Item) -> Result<String, DescriptionError> {
		use self::DescriptionError::*;
		let description: Cow<str> = match &self.description {
			Some(path) => {
				let description = util::read_to_string(project.path.join(path))?;
				if path.extension() == Some(OsStr::new("md")) {
					md_to_bb::convert(&description).into()
				} else {
					description.into()
				}
			},
			None => self
				.description_str
				.as_ref()
				.map(|s| s.as_ref())
				.unwrap_or("")
				.into(),
		};

		let description = if self.autodescription.unwrap_or(true) {
			let website: Cow<str> = if let Some(website) = &self.website {
				if let Some(commit) = project.head() {
					format!(
						"[b][url={}]website[/url][/b]\ncurrent git commit: {}\n\n",
						website, commit
					)
				} else {
					format!("[b][url={}]website[/url][/b]\n\n", website)
				}
				.into()
			} else {
				"".into()
			};

			let deps = project.dependencies().map_err(Dependencies)?;

			let deps = if deps.len() > 0 {
				let deps: Result<Vec<_>, DescriptionError> = deps
					.into_iter()
					.map(|d| match d.url() {
						Some(url) => Ok(Some(format!("  [*] [url={}]{}[/url]\n", url, d.name()?))),
						None => Ok(None),
					})
					.collect();

				let deps = deps?.into_iter().filter_map(|x| x).join_concat();

				format!("Mods included: [list]\n{}[/list]\n\n", deps)
			} else {
				"".into()
			};

			format!(
				"[b]Mod ID: {item:X}[/b]\n{website}{dependencies}\n{custom}",
				item = item.0,
				website = website,
				dependencies = deps,
				custom = description,
			)
		} else {
			description.into()
		};

		Ok(description)
	}

	pub fn preview(&self, project: &Project) -> Result<Vec<u8>, io::Error> {
		self.preview.as_ref().map_or(Ok(Vec::new()), |preview| {
			fs::read(project.path.join(preview))
		})
	}
}

pub struct Config {
	pub deps:              Vec<Item>,
	pub branches:          HashMap<String, Branch>,
	pub source_output_dir: Option<(PathBuf, PathBuf)>,
}

#[derive(Debug, Display, EError, From)]
pub enum ParseError {
	#[display(fmt = "{} is not a valid version!", _0)]
	InvalidVersion(i64),
	#[display(fmt = "Expected key {} of type {}", _0, _1)]
	ExpectedKey(&'static str, &'static str),
	#[display(fmt = "Could not parse ")]
	Error(#[error(source)] toml::de::Error),
}

#[derive(Debug, Display, EError, From)]
pub enum TOMLGetError {
	#[display(fmt = "Could not parse laspad.toml")]
	ParseError(#[error(source)] ParseError),
	#[display(fmt = "Could not read laspad.toml")]
	Read(#[error(source)] io::Error),
}

#[derive(Debug, Display, EError, From)]
pub enum LuaGetError {
	#[display(fmt = "Could not parse mod.settings")]
	Parse,
	#[display(fmt = "Could not read mod.settings")]
	Read(#[error(source)] io::Error),
	#[display(fmt = "Key '{}' is missing from mod.settings", _0)]
	MissingKey(&'static str),
	#[display(fmt = "'publish_id' field has an invalid format! It should be hexadecimal")]
	InvalidPublishId,
}

#[derive(Debug, Display, EError, From)]
pub enum GetError {
	#[display(fmt = "Could not get laspad.toml")]
	TOMLGetError(#[error(source)] TOMLGetError),
	#[display(fmt = "Could not get mod.settings")]
	LuaGetError(#[error(source)] LuaGetError),
}

fn parse_mod_settings(path: &Path) -> Result<Option<Config>, LuaGetError> {
	use self::LuaGetError::*;

	let conf = fs::read_to_string(path.join("mod.settings"));
	let conf = match conf.as_ref() {
		Ok(s) => static_lua::parse(s).map_err(|_| Parse)?,
		Err(ref e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
		Err(_) => return Err(conf.unwrap_err().into()),
	};

	let mut tags = Vec::new();

	for (k, &v) in &conf {
		if k.starts_with("tag_") {
			tags.push(v.into());
		}
	}

	let get = move |key: &'static str| conf.get(key).map(|s| *s).ok_or(MissingKey(key));

	let branch = Branch {
		name: get("name")?.into(),
		tags,
		autodescription: Some(false),
		description: None,
		description_str: Some(get("description")?.into()),
		preview: Some(get("image")?.into()),
		website: None,
		item: Some(get("publish_id")?.parse().map_err(|_| InvalidPublishId)?),
	};

	let mut branches = HashMap::new();
	branches.insert("master".into(), branch);

	Ok(Some(Config {
		source_output_dir: Some((get("source_dir")?.into(), get("output_dir")?.into())),
		deps: Vec::new(),
		branches,
	}))
}

impl Config {
	pub const EXAMPLE: Option<&'static str> = Some(include_str!("../assets/laspad.toml"));

	pub fn get(path: &Path) -> Result<Option<Self>, GetError> {
		match fs::read_to_string(path.join("laspad.toml")) {
			Ok(s) => Ok(Some(s.parse().map_err(TOMLGetError::ParseError)?)),
			Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(parse_mod_settings(path)?),
			Err(e) => Err(TOMLGetError::Read(e).into()),
		}
	}
}

impl FromStr for Config {
	type Err = ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use self::ParseError::*;

		let mut c: HashMap<String, toml::Value> = toml::de::from_str(s)?;

		let version = c.get("version").and_then(|v| v.as_integer()).map(|v| {
			c.remove("version");
			v
		});

		let c = match version {
			None | Some(0) => Config {
				source_output_dir: None,
				deps:              Vec::new(),
				branches:          c
					.into_iter()
					.map(|(k, v)| -> Result<_, toml::de::Error> {
						let v = v.try_into()?;
						Ok((k, v))
					})
					.collect::<Result<_, _>>()?,
			},
			Some(1) => Config {
				source_output_dir: None,
				deps:              c
					.remove("dependencies")
					.map_or(Ok(Vec::new()), |d| d.try_into())?,
				branches:          c
					.remove("branch")
					.ok_or(ExpectedKey("branch", "Table"))?
					.try_into()?,
			},
			Some(v) => return Err(InvalidVersion(v)),
		};
		Ok(c)
	}
}
