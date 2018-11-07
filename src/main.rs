#![feature(extern_types)]
#![feature(slice_concat_ext)]
#![allow(safe_packed_borrows)]
#![deny(unused_must_use)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate indoc;

mod common;
mod config;
mod md_to_bb;
mod steam;

// console commands
mod compile;
mod init;
mod launch;
mod need;
mod package;
mod prepare;
mod publish;
mod update;

use clap::{clap_app, crate_version};
use std::{process::exit, str::FromStr};

fn main() {
	let matches = clap_app!(laspad =>
		(version: crate_version!())
		(author:  "las <las@protonmail.ch>")
		(about:   "Replacement of Launch Pad for Natural Selection 2, i.e. can publish mods to workshop.")
		(@arg LOGLEVEL: -l --log +case_insensitive possible_value[off error warn info debug trace] "Sets the logging level")
		//(@setting SubcommandRequiredElseHelp)
		(@setting VersionlessSubcommands)
		(@subcommand init =>
		 	(about: "Initialises laspad in the current directory")
		)
		(@subcommand need =>
			(about: "Makes workshop item dependency")
			(@arg MODID: +required "Hexadecimal ID of workshop item")
		)
		(@subcommand update =>
			(about: "Updates dependencies\nNB: `publish` automatically runs `update`")
		)
		(@subcommand download =>
			(about: "Download and extract mod from workshop into target folder")
			(@arg MODID: +required "Hexadecimal ID of workshop item")
			(@arg PATH:  +required "Where to extract it")
		)
		(@subcommand compile =>
			(about: indoc!("
				Merges the dependencies and the `src` folder together into the `compiled` folder.
				NB: The files in the `compiled` folder are actually hard links.
				This means that changes in the compiled files will be reflected in the source and
				vice versa.
			"))
		)
		(@subcommand package =>
			(about: "Compiles the mod and then packages into a zip file which can be published")
			(@arg PATH: +required "Name of zip file generated")
			(@arg BRANCH: "The branch to package, defaults to master")
		)
		(@subcommand publish =>
			(about: "Updates dependencies and then publishes the mod to workshop")
			(@arg BRANCH: "The branch to publish, defaults to master")
			(@arg RETRY: -r --retry "Retry until success")
		)
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
	).get_matches();

	if let Err(e) = execute_command(&matches) {
		if cfg!(debug_assertions) {
			eprintln!("Fatal error: {:?}", e);
		} else {
			eprintln!("Fatal error: {}", e);
		};
		exit(1);
	};
}

fn execute_command<'a>(matches: &clap::ArgMatches<'a>) -> Result<(), failure::Error> {
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
				.map_or(log::LevelFilter::Info, |s| s.parse().unwrap()),
		)
		.chain(std::io::stderr())
		.apply()?;

	match matches.subcommand() {
		("", None) => unimplemented!("UI is unimplemented!"),
		("init", None) => init::main(),
		("need", Some(m)) => need::main(m.value_of("MODID").unwrap()),
		("update", Some(_)) => update::main(),
		("compile", Some(_)) => compile::main(),
		("package", Some(m)) => package::main(
			m.value_of("BRANCH").unwrap_or("master"),
			m.value_of("PATH").unwrap(),
		),
		("publish", Some(m)) => publish::main(
			m.value_of("BRANCH").unwrap_or("master"),
			m.is_present("RETRY"),
		),
		("launch", Some(m)) => launch::main(
			m.value_of("NS2ROOT"),
			launch::Program::from_str(m.subcommand_name().unwrap())?,
		),
		("prepare", Some(m)) => prepare::main(m.value_of("NS2ROOT")).map(|_| ()),
		("download", Some(m)) => {
			use std::fs;

			let path = m.value_of("PATH").unwrap();
			fs::create_dir_all(path)?;
			let modid = u64::from_str_radix(&m.value_of("MODID").unwrap().to_uppercase(), 16)?;
			update::specific(steam::Item(modid), m.value_of("PATH").unwrap())
		},
		_ => {
			unreachable!();
		},
	}
}
