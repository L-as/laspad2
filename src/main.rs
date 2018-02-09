#![feature(use_nested_groups)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

extern crate toml;
extern crate git2;
extern crate serde;
extern crate serde_xml_rs;
extern crate byteorder;
extern crate zip;
extern crate curl;
extern crate regex;
extern crate pretty_env_logger;

mod steam;

// console commands
mod init;
mod dependency;
mod compile;

use git2::Repository;

use std::process::exit;
use std::path::Path;

fn main() {
	//steam::init().unwrap_or_else(|_| {
	//	eprintln!("Could not initialise Steam API!");
	//	exit(1);
	//});

	pretty_env_logger::init();

	let matches = clap_app!(laspad =>
		(version: "2.0")
		(author:  "las <las@protonmail.ch>")
		(about:   "Replacement of Launch Pad for Natural Selection 2, i.e. can publish mods to workshop.")
		(@setting SubcommandRequiredElseHelp)
		(@setting VersionlessSubcommands)
		(@subcommand init =>
		 	(about: "Initialises laspad in the current directory")
		)
		(@subcommand dependency =>
			(about: "Manage dependencies")
			(@setting SubcommandRequiredElseHelp)
			(@setting VersionlessSubcommands)
			(@subcommand add =>
				(about: "Adds dependency")
				(@arg ID: +required "Hexadecimal ID of workshop item or link to git repository to add as dependency")
			)
			(@subcommand rm =>
				(about: "Removes dependency")
				(@arg ID: +required "Can be either a hexadecimal ID for a workshop item or the name of the repository to remove")
			)
		)
		(@subcommand compile =>
			(about: "\
Merges the dependencies and the `src` folder together into the `compiled` folder.
NB: The files in the `compiled` folder are actually hard links.
This means that changes in the compiled files will be reflected in the source and
vice versa.")
		)
		(@subcommand publish =>
			(about: "Publishes the mod to workshop")
			(@arg branch: "The branch to publish, defaults to master")
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

	let repo = Repository::open(".").unwrap_or_else(|e| {
		error!("Could not open git repository: {}", e); exit(1)
	});

	if matches.subcommand_name() == Some("init") {
		init::main(repo);
	} else {
		if !Path::new("laspad.toml").exists() {
			error!("This is not a laspad project!");
			exit(1);
		};

		match matches.subcommand() {
			("dependency", Some(m)) => {dependency::main(repo, m)},
			("compile", Some(_))    => {compile::main().unwrap()},
			_                       => {
				error!("Not a valid command!");
				exit(1);
			},
		};
	};
}
