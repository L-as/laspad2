#![feature(extern_types)]
#![feature(proc_macro, proc_macro_non_items)]
#![feature(slice_concat_ext)]

#![deny(unused_must_use)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

extern crate command_macros;

mod steam;
mod md_to_bb;
mod common;
mod builder;
mod config;

// console commands
mod init;
mod compile;
mod publish;
mod launch;
mod prepare;

use std::str::FromStr;

fn main() -> Result<(), failure::Error> {
	let mut builder = env_logger::Builder::from_default_env();

	if std::env::var_os("RUST_LOG").is_none() {
		builder.parse("laspad=info");
	}

	builder.init();

	let matches = clap_app!(laspad =>
		(version: crate_version!())
		(author:  "las <las@protonmail.ch>")
		(about:   "Replacement of Launch Pad for Natural Selection 2, i.e. can publish mods to workshop.")
		(@setting VersionlessSubcommands)
		(@setting SubcommandRequiredElseHelp)
		(@subcommand init =>
		 	(about: "Initialises laspad in the current directory")
			(@arg LUA: -l --lua "\
Recommended for advanced users.
Set this to generate a laspad project that uses a Lua configuration file instead.
Using Lua for configuration files allows you to customize the project much more,
including custom build rules.")
		)
		(@subcommand download =>
			(about: "Download and extract mod from workshop into target folder")
			(@arg MODID: +required "ID of workshop item")
			(@arg PATH:  +required "Where to place it")
		)
		(@subcommand compile =>
			(about: "\
Merges the dependencies and the `src` folder together into the `compiled` folder.
NB: The files in the `compiled` folder are actually hard links.
This means that changes in the compiled files will be reflected in the source and
vice versa.")
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

	match matches.subcommand() {
		("init",     Some(m)) =>    init::main(m.is_present("LUA")),
		("compile",  Some(_)) => compile::main(),
		("publish",  Some(m)) => publish::main(m.value_of("BRANCH").unwrap_or("master"), m.is_present("RETRY")),
		("launch",   Some(m)) =>  launch::main(m.value_of("NS2ROOT"), launch::Program::from_str(m.subcommand_name().unwrap())?),
		("prepare",  Some(m)) => prepare::main(m.value_of("NS2ROOT")).map(|_| ()),
		("download", Some(_m)) => {unimplemented!()},
		_ => {
			unreachable!();
		},
	}
}
