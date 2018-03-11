use std::{
	path::{self, Path, PathBuf},
	collections::HashMap,
	process::Command,
	fs,
};
use failure::*;
use common::*;

type Result<T> = ::std::result::Result<T, Error>;

// dst is relative to the output directory
type RuleHandler = fn(src: &Path, dst: &Path) -> Result<(Vec<PathBuf>, Command)>;

lazy_static! {
	static ref RULES: HashMap<&'static str, RuleHandler> = {
		let mut map = HashMap::new();
		let rules: &[(&'static str, RuleHandler)] = &[
			("level", |src, dst| Ok((
				{
					let mut s = dst.to_str().unwrap().to_owned();
					let sep   = path::MAIN_SEPARATOR;
					let pos   = s.as_bytes().iter().rposition(|&c| c == sep as u8).map_or(0, |p| p+1);
					s.insert_str(pos, "overviews/");
					let overview = PathBuf::from(s);
					vec![dst.to_owned(), overview.with_extension("tga"), overview.with_extension("hmp")]
				}, cmd!((get_ns2().join("Overview.exe")) (src) compiled)
			))),
		];
		for &(k, v) in rules.iter() {
			map.insert(k, v);
		};
		map
	};
}

pub struct Builder {
	new:        PathBuf,
	old:        Option<PathBuf>,
	rest:       Vec<(PathBuf, PathBuf)>,
	rest_built: bool,
}

impl Builder {
	pub fn new(new: PathBuf, old: Option<PathBuf>) -> Self {
		Builder {rest: Vec::new(), rest_built: false, new, old}
	}

	pub fn build(&mut self, src: &Path, dst: &Path) -> Result<()> {
		if let Some(rule) = src.extension().and_then(|s| s.to_str()).and_then(|s| RULES.get(s)) {
			let (outputs, mut cmd) = rule(src, dst)?;
			let old = self.old.as_ref();
			if old.map_or(true, |old| !outputs.iter().all(|o| old.join(o).exists())) {
				let status = cmd.status()?;
				if status.success() {
					Ok(())
				} else {
					Err(format_err!("Could not execute command {:?}, exit code: {}", cmd, status))
				}
			} else {
				for ref o in outputs {
					fs::hard_link(old.unwrap().join(o), self.new.join(o))?;
				};
				Ok(())
			}
		} else {
			self.rest.push((src.to_owned(), dst.to_owned()));
			Ok(())
		}
	}

	pub fn build_rest(&mut self) -> Result<()> {
		assert!(!self.rest_built);
		for &(ref src, ref dst) in self.rest.iter() {
			fs::hard_link(src, self.new.join(dst))?;
		};
		self.rest_built = true;
		Ok(())
	}
}


impl Drop for Builder {
	fn drop(&mut self) {
		assert!(self.rest_built);
	}
}
