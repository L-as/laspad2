use std::{
	path::Path,
	fs,
	ffi::OsStr,
};

use git2::Repository;
use failure::*;
use toml;
use rlua::{self, Lua};

use steam;
use md_to_bb;

type Result<T> = ::std::result::Result<T, Error>;

#[derive(Deserialize)]
struct TOMLBranch {
	name:            Box<str>,
	tags:            Vec<Box<str>>,
	autodescription: Option<bool>,
	description:     Option<Box<str>>,
	preview:         Option<Box<str>>,
	website:         Option<Box<str>>,
}

enum ConfigKind {
	TOML(toml::value::Table),
}
enum BranchKind {
	TOML(TOMLBranch),
}

pub struct Branch(BranchKind, steam::Item);
pub struct Config(ConfigKind);
impl<'a> Config {
	pub fn branches(&'a self) -> Vec<&'a str> {
		match self.0 {
			ConfigKind::TOML(ref table) => {
				table.keys().map(|s| s.as_str()).collect()
			}
		}
	}
	pub fn contains(&self, key: &str) -> bool {
		match self.0 {
			ConfigKind::TOML(ref table) => table.contains_key(key)
		}
	}
	pub fn get(&self, key: &str, item: steam::Item) -> Result<Option<Branch>> {
		match self.0 {
			ConfigKind::TOML(ref table) => {
				let v: TOMLBranch = if let Some(v) = table.get(key) {
					v.clone().try_into()?
				} else {
					return Ok(None)
				};
				Ok(Some(Branch(BranchKind::TOML(v), item)))
			}
		}
	}
}
impl Branch {
	pub fn name(&self) -> Result<&str> {
		match self.0 {
			BranchKind::TOML(ref branch) => Ok(&*branch.name)
		}
	}
	pub fn tags(&self) -> Result<&[Box<str>]> {
		match self.0 {
			BranchKind::TOML(ref branch) => Ok(&branch.tags)
		}
	}
	pub fn description(&self) -> Result<String> {
		match self.0 {
			BranchKind::TOML(ref toml) => {
				let description = if let Some(path) = toml.description.as_ref() {
					let description = fs::read_string(&**path).context("Could not read description")?;
					if Path::new(&**path).extension() == Some(OsStr::new("md")) {
						md_to_bb::convert(&description)
					} else {
						description
					}
				} else {
					Default::default()
				};

				let description = if toml.autodescription.unwrap_or(true) {
					let mut s = generate_autodescription(self.1, toml.website.as_ref().map(|b| b.as_ref()))?;
					s.push_str(&description);
					s
				} else {
					description
				};

				Ok(description)
			}
		}
	}
	pub fn preview(&self) -> Result<Vec<u8>> {
		match self.0 {
			BranchKind::TOML(ref branch) => {
				let mut preview = if let Some(preview) = branch.preview.as_ref() {
					fs::read(&**preview).context("Could not read preview")?
				} else {
					Default::default()
				};
				if preview.len() == 0 { // Steam craps itself when it has 0 length
					preview.extend_from_slice(b"\x89PNG\r\n\x1A\n"); // PNG header so that it shows an empty image in browsers instead of an error
				};
				Ok(preview)
			}
		}
	}
}

fn generate_autodescription(item: steam::Item, website: Option<&str>) -> Result<String> {
	let mut s: String = format!(
		"[b]Mod ID: {}[/b]\n\n",
		item
	);

	if Path::new(".git").exists() && website.is_some() {
		let repo = Repository::open(".")?;
		let head   = repo.head()?;
		let oid    = head.peel_to_commit()?.id();
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
		for dependency in fs::read_dir("dependencies").context("Couldn't read dependencies directory")? {
			let dependency = dependency?;
			let path       = dependency.path();
			let name       = dependency.file_name().into_string().expect("Invalid UTF-8");
			let (name, url) = if let Ok(modid) = u64::from_str_radix(&name, 16) {
				#[derive(Deserialize)]
				struct ModInfo {
					name: Box<str>
				}

				let s = fs::read_string(path.join(".modinfo")).with_context(|_| format!("Couldn't read .modinfo file for {}", &name))?;
				let modinfo: ModInfo = toml::from_str(&s)?;

				let url = format!("http://steamcommunity.com/sharedfiles/filedetails/?id={}", modid);

				(modinfo.name, url.into_boxed_str())
			} else if path.join(".git").exists() {
				let repo   = Repository::open(path)?;
				let origin = repo.find_remote("origin")?;
				let url    = origin.url().unwrap();

				(name.into_boxed_str(), String::from(url).into_boxed_str())
			} else {
				continue
			};
			s.push_str(&format!(
				"  [*] [url={}]{}[/url]\n",
				url,
				name
			));
		};
		s.push_str("[/list]\n\n");
	};

	Ok(s)
}

pub fn get() -> Result<Config> {
	let toml = Path::new("laspad.toml").exists();
	let lua  = Path::new("laspad.lua").exists();
	ensure!(!lua || !toml, "You can not use both Lua *and* TOML configuration files!");
	if lua {
		unimplemented!()
	} else if toml {
		let toml: toml::Value = fs::read_string("laspad.toml")?.parse()?;
		let toml = if let toml::Value::Table(t) = toml {
			t
		} else {
			bail!("The TOML configuration file has to be a table!");
		};
		Ok(Config(ConfigKind::TOML(toml)))
	} else {
		unreachable!();
	}
}
