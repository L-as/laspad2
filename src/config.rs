use std::{
	path::Path,
	fs,
	ffi::OsStr,
	borrow::Cow,
	result,
};

use git2::Repository;
use failure::*;
use toml;
use rlua::{FromLua, self, prelude::*};

use steam;
use md_to_bb;

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
	// WARNING! Don't drop the Lua instance without dropping the table too!
	// The table isn't static either actually
	Lua(Box<Lua>, LuaTable<'static>),
	TOML(toml::value::Table),
}
enum BranchKind<'a> {
	Lua(&'a Lua, LuaTable<'a>),
	TOML(TOMLBranch),
}

pub struct Branch<'a>(BranchKind<'a>, steam::Item);
pub struct Config(ConfigKind);
impl<'a> Config {
	pub fn branches(&'a self) -> Result<Vec<Cow<'a, str>>> {
		match self.0 {
			ConfigKind::TOML(ref table) => {
				Ok(table.keys().map(|s| Cow::Borrowed(s.as_str())).collect())
			},
			ConfigKind::Lua(ref _lua, ref table) => {
				let r: Vec<_> = table.clone().pairs::<String, LuaValue>().map(|r| match r {
					Ok((k, _v)) => Ok(Cow::Owned(k)),
					Err(e)     => Err(e),
				}).collect::<result::Result<_, _>>()?;
				Ok(r)
			},
		}
	}
	pub fn contains(&self, key: &str) -> bool {
		match self.0 {
			ConfigKind::TOML(ref table) => table.contains_key(key),
			ConfigKind::Lua(ref _lua, ref table) => {
				table.contains_key(key).unwrap_or(false)
			},
		}
	}
	pub fn get(&'a self, key: &str, item: steam::Item) -> Result<Option<Branch<'a>>> {
		match self.0 {
			ConfigKind::TOML(ref table) => {
				let v: TOMLBranch = if let Some(v) = table.get(key) {
					v.clone().try_into()?
				} else {
					return Ok(None)
				};
				Ok(Some(Branch(BranchKind::TOML(v), item)))
			},
			ConfigKind::Lua(ref lua, ref table) => {
				let v: LuaTable = table.get(key)?;
				Ok(Some(Branch(BranchKind::Lua(lua, v), item)))
			},
		}
	}
}

fn get_value_lua<'a, T: FromLua<'a>>(lua: &'a Lua, item: steam::Item, table: &LuaTable<'a>, key: &str) -> Result<T> {
	let v = table.get(key)?;
	match v {
		LuaValue::Function(f) => Ok(f.call(item.0)?),
		e => Ok(T::from_lua(e, lua)?),
	}
}

impl<'a> Branch<'a> {
	pub fn name(&self) -> Result<Cow<str>> {
		match self.0 {
			BranchKind::TOML(ref branch)        => Ok(Cow::Borrowed(&branch.name)),
			BranchKind::Lua(ref lua, ref table) => Ok(Cow::Owned(get_value_lua(lua, self.1, table, "name")?)),
		}
	}
	pub fn tags(&self) -> Result<Cow<[String]>> {
		match self.0 {
			BranchKind::TOML(ref branch)        => Ok(Cow::Borrowed(&branch.tags)),
			BranchKind::Lua(ref lua, ref table) => Ok(Cow::Owned(get_value_lua(lua, self.1, table, "tags")?)),
		}
	}
	pub fn description(&self) -> Result<String> {
		match self.0 {
			BranchKind::TOML(ref toml)           => read_description(
				toml.description.as_ref().map(|s| s.as_ref()),
				toml.autodescription.unwrap_or(false),
				toml.website.as_ref().map(|s| s.as_ref()),
				self.1),
			BranchKind::Lua (ref lua, ref table) => Ok(get_value_lua(lua, self.1, table, "description")?),
		}
	}
	pub fn preview(&self) -> Result<Vec<u8>> {
		fn default(mut v: Vec<u8>) -> Vec<u8> {
			if v.len() == 0 { // Steam craps itself when it has 0 length
				v.extend_from_slice(b"\x89PNG\r\n\x1A\n"); // PNG header so that it shows an empty image in browsers instead of an error
			};
			v
		}
		match self.0 {
			BranchKind::TOML(ref branch) => {
				let mut preview = if let Some(preview) = branch.preview.as_ref() {
					fs::read(preview).context("Could not read preview")?
				} else {
					Default::default()
				};
				Ok(default(preview))
			},
			BranchKind::Lua(ref lua, ref table) => Ok(default(get_value_lua::<rlua::String>(lua, self.1, table, "preview")?.as_bytes().to_owned())),
		}
	}
}

fn read_description(path: Option<&Path>, auto_description: bool, website: Option<&str>, item: steam::Item) -> Result<String> {
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

				let s = fs::read_to_string(path.join(".modinfo")).with_context(|_| format!("Couldn't read .modinfo file for {}", &name))?;
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

fn lua_stdlib(lua: &Lua) -> Result<()> {
	use std::sync::Arc;

	let globals = lua.globals();
	let stdlib = lua.create_table()?;

	stdlib.set("get_description", lua.create_function(|lua, (path, auto_description, website): (String, Option<bool>, Option<String>)| {
		lua.create_function(move |_, item: u64| {
			let path = Path::new(&path);
			let item = steam::Item(item);
			read_description(Some(path), auto_description.unwrap_or(true), website.as_ref().map(|s| s.as_ref()), item).map_err(|e| {
				LuaError::ExternalError(Arc::new(e))
			})
		})
	})?)?;

	stdlib.set("read", lua.create_function(|lua, path: String| {
		lua.create_function(move |_, _item: u64| {
			// Not actually valid UTF-8
			// We just do this because otherwise rlua won't convert it to a lua string
			fs::read(&path).map(|p| unsafe {String::from_utf8_unchecked(p)}).map_err(|e| {
				LuaError::ExternalError(Arc::new(e.into()))
			})
		})
	})?)?;

	globals.set("laspad", stdlib)?;
	Ok(())
}

pub fn get() -> Result<Config> {
	use std::mem::transmute;

	let toml = Path::new("laspad.toml").exists();
	let lua  = Path::new("laspad.lua").exists();
	ensure!(!lua || !toml, "You can not use both Lua *and* TOML configuration files!");
	if lua {
		let lua = Box::new(Lua::new());
		lua_stdlib(&lua)?;
		let table: LuaTable<'static> = {
			let table: LuaTable = lua.exec(&fs::read_to_string("laspad.lua")?, Some("laspad.lua"))?;
			unsafe {transmute(table)}
		};
		Ok(Config(ConfigKind::Lua(lua, table)))
	} else if toml {
		let toml: toml::Value = fs::read_to_string("laspad.toml")?.parse()?;
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
