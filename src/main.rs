#![feature(extern_types)]
#![feature(fs_read_write)]
#![allow(safe_packed_borrows)]
#![deny(unused_must_use)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure;

extern crate toml;
extern crate serde;
extern crate serde_xml_rs;
extern crate byteorder;
extern crate zip;
extern crate curl;
extern crate regex;
extern crate git2;
extern crate web_view as webview;
extern crate urlencoding;
extern crate nfd;
extern crate termcolor;

mod steam;
mod md_to_bb;
mod ui;
mod common;
#[macro_use]
mod logger;

// console commands
mod init;
mod need;
mod update;
mod compile;
mod publish;

use std::process::exit;
use std::io::Write;
use std::cell::RefCell;
use std::env;

use termcolor::{StandardStream, ColorChoice, ColorSpec, Color, WriteColor};

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

	struct StdLog<'a> {
		min_priority: i64,
		stdout: RefCell<&'a mut StandardStream>,
		stderr: RefCell<&'a mut StandardStream>,
	}

	impl<'a> logger::Log for StdLog<'a> {
		fn write(&self, priority: i64, line: &str) {
			if priority < self.min_priority {return};

			if priority > 0 {
				let mut s = self.stderr.borrow_mut();
				let _ = s.set_color(ColorSpec::new().set_fg(Some(Color::Red)));
				let _ = writeln!(s, "{}", line);
				let _ = s.reset();
			} else if priority == 0 {
				let mut s = self.stdout.borrow_mut();
				let _ = s.set_color(ColorSpec::new().set_bold(true));
				let _ = writeln!(s, "{}", line);
				let _ = s.reset();
			} else {
				let _ = writeln!(self.stdout.borrow_mut(), "{}", line);
			};
		}
	}

	let mut stdout = StandardStream::stdout(ColorChoice::Auto);
	let mut stderr = StandardStream::stderr(ColorChoice::Auto);

	let log = &mut StdLog {
		stdout: RefCell::new(&mut stdout),
		stderr: RefCell::new(&mut stderr),
		min_priority: -(matches.occurrences_of("VERBOSITY") as i64 + 1),
	};

	if let Err(e) = execute_command(&matches, log) {
		elog!(log; "Fatal error: {}", e);
		exit(1);
	};
}

fn execute_command<'a>(matches: &clap::ArgMatches<'a>, log: &logger::Log) -> Result<(), failure::Error> {
	match matches.subcommand() {
		("",         None)    =>      ui::main(),
		("init",     Some(_)) =>    init::main(log),
		("need",     Some(m)) =>    need::main(m.value_of("MODID" ).unwrap(), log),
		("update",   Some(_)) =>  update::main(log),
		("compile",  Some(_)) => compile::main(log),
		("publish",  Some(m)) => publish::main(m.value_of("BRANCH").unwrap_or("master"), m.is_present("RETRY"), log),
		("download", Some(m)) => {
			use std::fs;

			let path = m.value_of("PATH").unwrap();
			fs::create_dir_all(path)?;
			let modid = u64::from_str_radix(&m.value_of("MODID").unwrap().to_uppercase(), 16)?;
			update::specific(steam::Item(modid), m.value_of("PATH").unwrap(), log)
		},
		_ => {
			unreachable!();
		},
	}
}
