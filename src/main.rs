#[macro_use]
extern crate log;

mod compile;
mod config;
mod download;
mod item;
mod package;
mod project;
mod publish;
mod util;

use clap::{clap_app, crate_version};
use derive_more::{Display, From};
use erroneous::Error as EError;
use std::{
	fmt,
	fs::{self, File},
	io,
	path::{Path, PathBuf},
};

use self::{
	item::{Item, ItemParseError},
	project::Project,
};

#[derive(Display, EError, From)]
enum Error {
	#[display(fmt = "Could not check if there is an existing laspad project")]
	ProjectGetError(#[error(source)] project::GetError),
	#[display(fmt = "Could not create a new laspad project")]
	ProjectNewError(#[error(source)] project::NewError),
	#[display(fmt = "This is not a laspad project!")]
	NoProject,
	#[display(fmt = "Could not download workshop item")]
	DownloadError(#[error(source)] download::Error),
	#[display(fmt = "Could not compile project")]
	CompileError(#[error(source)] compile::Error),
	#[display(fmt = "Could not update steam dependencies")]
	ProjectUpdateError(#[error(source)] project::UpdateError),
	#[display(fmt = "Could not parse '{}'", _0)]
	ItemParseError(String, #[error(source)] ItemParseError),
	#[display(fmt = "Could not create specified file")]
	CreateFile(#[error(source)] io::Error),
	#[display(fmt = "Could not package project into an archive")]
	PackageError(#[error(source)] package::Error),
	#[display(fmt = "Could not publish project")]
	PublishError(#[error(source)] publish::Error),
	#[display(fmt = "Could not remove existing 'compiled' directory")]
	RemoveCompiled(#[error(source)] io::Error),
}

impl fmt::Debug for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut errors = self.iter();
		if let Some(error) = errors.next() {
			fmt::Display::fmt(error, f)?;
		}
		for error in errors {
			write!(f, ": ")?;
			fmt::Display::fmt(error, f)?;
		}
		Ok(())
	}
}

fn main() -> Result<(), Error> {
	let matches = clap_app!(laspad =>
		(version: crate_version!())
		(author:  "las <las@protonmail.ch>")
		(about:   "Replacement of Launch Pad for Natural Selection 2, i.e. can publish mods to workshop.")
		(@arg LOGLEVEL: -l --log +takes_value +case_insensitive possible_value[off error warn info debug trace] "Sets the logging level")
		(@setting SubcommandRequiredElseHelp)
		(@setting VersionlessSubcommands)
		(@subcommand init =>
		 	(about: "Initialises laspad in the current directory")
		)
		(@subcommand update =>
			(about: "Updates dependencies")
			(@arg ITEMS: #{0, u64::max_value()} "Steam items to update, none will mean all")
		)
		(@subcommand download =>
			(about: "Download and extract mod from workshop into target folder")
			(@arg MODID: +required "The workshop item")
			(@arg PATH:  +required "Where to extract it")
		)
		(@subcommand compile =>
			(about: "\
Merges the dependencies and the `src` folder together into the `compiled` folder.
NB: The files in the `compiled` folder are actually hard links.
This means that changes in the compiled files will be reflected in the source and
vice versa.")
		)
		(@subcommand package =>
			(about: "Compiles the mod and then packages into a zip file which can be published")
			(@arg PATH: +required "Name of zip file generated")
			(@arg BRANCH: "The branch to package, defaults to master")
		)
		(@subcommand publish =>
			(about: "Updates dependencies and then publishes the mod to workshop")
			(@arg BRANCH: "The branch to publish, defaults to master")
		)
		/* FIXME
		(@subcommand prepare =>
			(about: "Runs `compile` and allows you to launch any Spark program with this mod by passing `-game laspad_mod` to it")
			(@arg NS2ROOT: +takes_value -r --root "The root of the NS2 installation directory")
		)
		(@subcommand launch =>
			(about: "Launches an external spark program with this mod")
			(@setting SubcommandRequiredElseHelp)
			(@setting VersionlessSubcommands)
			(@arg NS2ROOT: +takes_value -r --root "The root of the NS2 installation directory")
			(@subcommand ns2 =>
				(about: "Launches NS2 with this mod, making it active for any map you launch (local or remote), useful for testing")
			)
			(@subcommand editor =>
				(about: "Launches Editor with this mod active (allows you to use entities from this mod)")
			)
		)
		*/
	).get_matches();

	fern::Dispatch::new()
		.format(|out, message, record| {
			out.finish(format_args!(
				"[{}][{}] {}",
				record.target(),
				record.level(),
				message
			))
		})
		.level(
			matches
				.value_of("LOGLEVEL")
				.map_or(log::LevelFilter::Info, |s| s.parse().expect("Could not parse LOGLEVEL")),
		)
		.chain(std::io::stderr())
		.apply()
		.expect("Could not initiate logging");

	// FIXME
	let mut path: &Path = &Path::canonicalize(".".as_ref()).unwrap();
	let project = loop {
		if let Some(project) = Project::get(path)? {
			break Some(project);
		}

		path = match path.parent() {
			Some(p) => p,
			None => break None,
		};
	};

	match matches.subcommand() {
		("", _) => unimplemented!("UI is unimplemented!"),
		("init", _) => {
			Project::new(".")?;
		},
		("download", Some(m)) => {
			let item = m.value_of("MODID").expect("Could not get MODID");
			let item: Item = item.parse().map_err(|e| (item.to_owned(), e))?;
			let path = m.value_of("PATH").expect("Could not get PATH");
			download::download(item, path)?;
		},
		(cmd, m) => {
			let project = project.ok_or(Error::NoProject)?;
			match (cmd, m) {
				("update", m) => {
					match m.and_then(|m| m.values_of("ITEMS")) {
						Some(items) => {
							let items: Result<Vec<_>, _> = items
								.map(|i| {
									i.parse()
										.map_err(|e| Error::ItemParseError(i.to_owned(), e))
								})
								.collect();
							project.update(items?)?;
						},
						None => project.update(project.config.deps.clone())?,
					};
				},
				("compile", _) => {
					struct Target {
						path: PathBuf,
					}
					impl compile::Out for Target {
						fn dir(&mut self, path: &Path) -> Result<(), io::Error> {
							fs::create_dir_all(self.path.join(path))
						}

						fn file(&mut self, src: &Path, dst: &Path) -> Result<(), io::Error> {
							let dst = self.path.join(dst);
							if dst.exists() {
								fs::remove_file(&dst)?;
							}
							fs::hard_link(src, dst)
						}
					}
					let mut out = Target {
						path: project.path.join("compiled"),
					};
					fs::remove_dir_all(&out.path).map_err(Error::RemoveCompiled)?;
					compile::compile(&project, &mut out)?;
				},
				("package", Some(m)) => {
					let branch = m.value_of("BRANCH").unwrap_or("master");
					let path = m.value_of("PATH").expect("Could not get PATH");
					let file = File::create(path).map_err(Error::CreateFile)?;
					package::package(&project, &project.config.branches[branch], file)?;
				},
				("publish", Some(m)) => {
					let branch = m.value_of("BRANCH").unwrap_or("master");
					publish::publish(&project, &project.config.branches[branch], &branch)?;
				},
				_ => {
					unreachable!();
				},
			}
		},
	};

	Ok(())
}
