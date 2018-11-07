use failure::*;
use git2::Repository;
use serde_derive::Deserialize;
use std::{borrow::Cow, ffi::OsStr, fs, path::Path};
use toml;

use crate::{md_to_bb, steam};

type Result<T> = ::std::result::Result<T, Error>;

#[derive(Deserialize)]
struct TOMLBranch {
	name:            String,
	tags:            Vec<String>,
	autodescription: Option<bool>,
	description:     Option<String>,
	preview:         Option<String>,
	website:         Option<String>,
}

enum ConfigKind {
	TOML(toml::value::Table),
}
enum BranchKind {
	TOML(TOMLBranch),
}

pub struct Branch(BranchKind);
pub struct Config(ConfigKind);
impl<'a> Config {
	pub fn branches(&'a self) -> Result<Vec<Cow<'a, str>>> {
		match self.0 {
			ConfigKind::TOML(ref table) => {
				Ok(table.keys().map(|s| Cow::Borrowed(s.as_str())).collect())
			},
		}
	}

	pub fn contains(&self, key: &str) -> bool {
		match self.0 {
			ConfigKind::TOML(ref table) => table.contains_key(key),
		}
	}

	pub fn get(&'a self, key: &str) -> Result<Option<Branch>> {
		log!(2; "Accessed branch {}", key);
		match self.0 {
			ConfigKind::TOML(ref table) => {
				let v: TOMLBranch = if let Some(v) = table.get(key) {
					v.clone().try_into()?
				} else {
					return Ok(None);
				};
				Ok(Some(Branch(BranchKind::TOML(v))))
			},
		}
	}
}

impl Branch {
	pub fn name(&self) -> Result<Cow<'_, str>> {
		match self.0 {
			BranchKind::TOML(ref branch) => Ok(Cow::Borrowed(&branch.name)),
		}
	}

	pub fn tags(&self) -> Result<Cow<'_, [String]>> {
		match self.0 {
			BranchKind::TOML(ref branch) => Ok(Cow::Borrowed(&branch.tags)),
		}
	}

	pub fn description(&self, item: steam::Item) -> Result<String> {
		match self.0 {
			BranchKind::TOML(ref toml) => read_description(
				toml.description.as_ref().map(|s| s.as_ref()),
				toml.autodescription.unwrap_or(false),
				toml.website.as_ref().map(|s| s.as_ref()),
				item,
			),
		}
	}

	pub fn preview(&self) -> Result<Vec<u8>> {
		fn default(mut v: Vec<u8>) -> Vec<u8> {
			if v.len() == 0 {
				// Steam craps itself when it has 0 length
				v.extend_from_slice(b"\x89PNG\r\n\x1A\n"); // PNG header so that it shows an empty image in browsers instead of an error
			};
			v
		}
		match self.0 {
			BranchKind::TOML(ref branch) => {
				let preview = if let Some(preview) = branch.preview.as_ref() {
					fs::read(preview).context("Could not read preview")?
				} else {
					Default::default()
				};
				Ok(default(preview))
			},
		}
	}
}

fn read_description(
	path: Option<&Path>,
	auto_description: bool,
	website: Option<&str>,
	item: steam::Item,
) -> Result<String> {
	let description = match path {
		Some(path) => {
			let description = fs::read_to_string(path).context("Could not read description")?;
			if path.extension() == Some(OsStr::new("md")) {
				md_to_bb::convert(&description)
			} else {
				description
			}
		},
		None => Default::default(),
	};

	let description = if auto_description {
		let mut s = generate_autodescription(item, website)?;
		s.push_str(&description);
		s
	} else {
		description
	};

	Ok(description)
}

fn generate_autodescription(item: steam::Item, website: Option<&str>) -> Result<String> {
	let mut s: String = format!("[b]Mod ID: {:X}[/b]\n\n", item.0);

	if Path::new(".git").exists() && website.is_some() {
		let repo = Repository::open(".")?;
		let head = repo.head()?;
		let oid = head.peel_to_commit()?.id();
		s.push_str(&format!(
			"[b][url={}]git repository[/url][/b]\ncurrent git commit: {}\n\n",
			website.unwrap(),
			oid
		));
	} else if website.is_some() {
		s.push_str(&format!(
			"[b][url={}]website[/url][/b]\n\n",
			website.unwrap()
		));
	};

	if Path::new("dependencies").exists() {
		s.push_str("Mods included: [list]\n");
		for dependency in
			fs::read_dir("dependencies").context("Couldn't read dependencies directory")?
		{
			let dependency = dependency?;
			let path = dependency.path();
			let name = dependency.file_name().into_string().expect("Invalid UTF-8");
			let (name, url) = if let Ok(modid) = u64::from_str_radix(&name, 16) {
				#[derive(Deserialize)]
				struct ModInfo {
					name: Box<str>,
				}

				let s = fs::read_to_string(path.join(".modinfo"))
					.with_context(|_| format!("Couldn't read .modinfo file for {}", &name))?;
				let modinfo: ModInfo = toml::from_str(&s)?;

				let url = format!(
					"http://steamcommunity.com/sharedfiles/filedetails/?id={}",
					modid
				);

				(modinfo.name, url.into_boxed_str())
			} else if path.join(".git").exists() {
				let repo = Repository::open(path)?;
				let origin = repo.find_remote("origin")?;
				let url = origin.url().unwrap();

				(name.into_boxed_str(), String::from(url).into_boxed_str())
			} else {
				continue;
			};
			s.push_str(&format!("  [*] [url={}]{}[/url]\n", url, name));
		}
		s.push_str("[/list]\n\n");
	};

	Ok(s)
}

pub fn get() -> Result<Config> {
	log!(2; "Reading laspad.toml");
	let toml: toml::Value = fs::read_to_string("laspad.toml")?.parse()?;
	let toml = if let toml::Value::Table(t) = toml {
		t
	} else {
		bail!("The TOML configuration file has to be a table!");
	};
	Ok(Config(ConfigKind::TOML(toml)))
}
