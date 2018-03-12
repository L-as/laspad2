use std::{
	path::{self, Path, PathBuf},
	process::Command,
	fs,
};
use failure::*;
use common::*;

type Result<T> = ::std::result::Result<T, Error>;

struct Rule {
	outputs: Vec<PathBuf>,
	cmd:     Command,
}

pub struct Builder {
	new:        PathBuf,
	old:        Option<PathBuf>,
	rest:       Vec<(PathBuf, PathBuf)>,
	rest_built: bool,
}

impl Builder {
	fn get_rule(&mut self, src: &Path, dst: &Path) -> Option<Rule> {
		if let Some(extension) = src.extension().and_then(|s| s.to_str()) {
			match extension {
				"level" => {
					let mut s = dst.to_str().unwrap().to_owned();
					let sep   = path::MAIN_SEPARATOR;
					let pos   = s.as_bytes().iter().rposition(|&c| c == sep as u8).map_or(0, |p| p+1);
					s.insert_str(pos, "overviews/");
					let overview = PathBuf::from(s);
					Some(Rule {
						outputs: vec![dst.to_owned(), overview.with_extension("tga"), overview.with_extension("hmp")],
						cmd:     cmd!((get_ns2().join("Overview.exe")) (src) compiled),
					})
				},
				"psd"   => {
					let dst = dst.with_extension("dds");
					let cmd = if src.ends_with("_normal.psd") {
						cmd!((get_ns2().join("../utils/nvcompress")) (-normal) (-bc1) (src) (self.new.join(&dst)))
					} else {
						cmd!((get_ns2().join("../utils/nvcompress"))           (-bc3) (src) (self.new.join(&dst)))
					};
					let outputs = vec![dst];

					Some(Rule {
						outputs: outputs,
						cmd:     cmd,
					})
				},
				_ => None,
			}
		} else {
			None
		}
	}

	pub fn new(new: PathBuf, old: Option<PathBuf>) -> Self {
		Builder {rest: Vec::new(), rest_built: false, new, old}
	}

	pub fn build(&mut self, src: &Path, dst: &Path) -> Result<()> {
		if let Some(Rule {outputs, mut cmd}) = self.get_rule(src, dst) {
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
		self.rest_built = true;
		for &(ref src, ref path) in self.rest.iter() {
			let dst = &self.new.join(path);
			if dst.exists() {fs::remove_file(dst)?};
			fs::hard_link(src, dst)?;
		};
		Ok(())
	}
}


impl Drop for Builder {
	fn drop(&mut self) {
		assert!(self.rest_built);
	}
}
