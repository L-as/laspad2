#![feature(extern_types)]
#![allow(safe_packed_borrows)]
#![deny(unused_must_use)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

extern crate toml;
extern crate serde;
extern crate serde_xml_rs;
extern crate byteorder;
extern crate zip;
extern crate curl;
extern crate regex;
extern crate pretty_env_logger;
extern crate git2;

mod steam;

// console commands
mod init;
mod need;
mod update;
mod compile;
mod publish;
mod md_to_bb;

use std::process::exit;
use std::path::Path;

fn main() {
	//steam::init().unwrap_or_else(|_| {
	//	eprintln!("Could not initialise Steam API!");
	//	exit(1);
	//});

	pretty_env_logger::init();

	let matches = clap_app!(laspad =>
		(version: crate_version!())
		(author:  "las <las@protonmail.ch>")
		(about:   "Replacement of Launch Pad for Natural Selection 2, i.e. can publish mods to workshop.")
		(@setting SubcommandRequiredElseHelp)
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
		//(@subcommand launch =>
		// 	(about: "Launches an external spark program with this mod")
		//	(@setting SubcommandRequiredElseHelp)
		//	(@setting VersionlessSubcommands)
		//	(@subcommand ns2 =>
		//		(about: "Launches NS2")
		//	(@subcommand editor =>
		//		(about: "Launches Editor")
		//	)
		//	(@subcommand builder =>
		//		(about: "Launches Builder")
		//	)
		//)
	).get_matches();

	if matches.subcommand_name() == Some("init") {
		init::main().unwrap();
	} else {
		while !Path::new("laspad.toml").exists() {
			use std::env;
			if let Some(parent) = env::current_dir().unwrap().parent() {
				env::set_current_dir(&parent).unwrap();
			} else {
				error!("This is not a laspad project!");
				exit(1);
			};
		};

		match matches.subcommand() {
			("need",    Some(m)) => {need::   main(m.value_of("MODID" ).unwrap()).unwrap()},
			("update",  Some(_)) => {update:: main().unwrap()},
			("compile", Some(_)) => {compile::main().unwrap()},
			("publish", Some(m)) => {publish::main(m.value_of("BRANCH").unwrap_or("master"), m.is_present("RETRY"))},
			_                       => {
				unreachable!();
			},
		};
	};
}
