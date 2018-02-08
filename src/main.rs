#![feature(use_nested_groups)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

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

use git2::Repository;

use std::process::exit;

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
		(@subcommand init =>
		 	(about: "Initialises laspad in the current directory")
		)
		(@subcommand dependency =>
			(about: "Manage dependencies")
			(@subcommand add =>
				(@arg ID: +required "Hexadecimal ID of workshop item or link to git repository to add as dependency")
			)
			(@subcommand rm =>
				(@arg ID: +required "Can be either a hexadecimal ID for a workshop item or the name of the repository to remove")
			)
		)
	).get_matches();

	let repo = Repository::open(".").unwrap_or_else(|e| {
		eprintln!("Could not open git repository: {}", e); exit(1)
	});

	match matches.subcommand() {
		("init",       _)       => {init::main(repo)},
		("dependency", Some(m)) => {dependency::main(repo, m)},
		_                       => {
			eprintln!("Not a valid command!");
			exit(1);
		},
	};
}
