use git2::Repository;

use std::process::exit;
use std::fs::{self, File};
use std::io::Write;

pub fn main(repo: Repository) {
	if File::open("laspad.toml").is_ok() {
		eprintln!("This is already a laspad project!");
		exit(1);
	};

	File::create("laspad.toml")
		.expect("Could not create laspad.toml!")
		.write_all(include_bytes!("../laspad.toml"))
		.expect("Could not create laspad.toml!");

	println!("Example laspad.toml created. Please modify it.");

	fs::create_dir_all("src").unwrap_or_else(|e| eprintln!("Could not create src directory: {}", e));

	repo.add_ignore_rule("compiled").unwrap_or_else(|e| {
		eprintln!("Could not add 'compiled' to .gitignore: {}", e);
	});
}
