#![feature(extern_types)]
#![feature(fs_read_write)]
#![feature(slice_concat_ext)]
#![allow(safe_packed_borrows)]
#![deny(unused_must_use)]
#![windows_subsystem = "windows"]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate downcast;

extern crate toml;
extern crate serde;
extern crate serde_xml_rs;
extern crate byteorder;
extern crate zip;
extern crate curl;
extern crate regex;
extern crate git2;
extern crate web_view;
extern crate termcolor;
extern crate futures;
extern crate hyper;
extern crate mime;

#[macro_use]
mod logger;

mod steam;
mod md_to_bb;
mod ui;
mod common;

// console commands
mod init;
mod need;
mod update;
mod compile;
mod publish;

use std::process::exit;
use std::io::Write;
use std::env;

use termcolor::{StandardStream, ColorChoice, ColorSpec, Color, WriteColor};

use logger::Log;

struct StdLog {
	stdout: StandardStream,
	stderr: StandardStream,
}

impl Log for StdLog {
	fn write(&mut self, priority: i64, line: &str) {
		if priority > 0 {
			let _ = self.stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)));
			let _ = writeln!(self.stderr, "{}", line);
			let _ = self.stderr.reset();
		} else if priority == 0 {
			let _ = self.stdout.set_color(ColorSpec::new().set_bold(true));
			let _ = writeln!(self.stdout, "{}", line);
			let _ = self.stdout.reset();
		} else {
			let _ = writeln!(self.stdout, "{}", line);
		};
	}
}

fn main() {
	if env::var_os("RUST_LOG").is_none() {
		env::set_var("RUST_LOG", "laspad=info")
	};

	let matches = clap_app!(laspad =>
		(version: crate_version!())
		(author:  "las <las@protonmail.ch>")
		(about:   "Replacement of Launch Pad for Natural Selection 2, i.e. can publish mods to workshop.")
		(@arg VERBOSITY: -v +multiple "Sets verbosity, use multiple times to increase verbosity.")
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

	logger::set_priority(-(matches.occurrences_of("VERBOSITY") as i64 + 1));

	let log = StdLog {
		stdout: StandardStream::stdout(ColorChoice::Auto),
		stderr: StandardStream::stderr(ColorChoice::Auto),
	};

	logger::set(Box::new(log));

	if let Err(e) = execute_command(&matches) {
		elog!(log; "Fatal error: {}", e);
		exit(1);
	};
}

fn execute_command<'a>(matches: &clap::ArgMatches<'a>) -> Result<(), failure::Error> {
	match matches.subcommand() {
		("",         None)    =>      ui::main(),
		("init",     Some(_)) =>    init::main(),
		("need",     Some(m)) =>    need::main(m.value_of("MODID" ).unwrap()),
		("update",   Some(_)) =>  update::main(),
		("compile",  Some(_)) => compile::main(),
		("publish",  Some(m)) => publish::main(m.value_of("BRANCH").unwrap_or("master"), m.is_present("RETRY")),
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
