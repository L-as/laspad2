use git2::Repository;

use clap::ArgMatches;

use byteorder::{LE, ReadBytesExt, WriteBytesExt};

use serde_xml_rs;

use std::fs::{self, File};
use std::process::exit;
use std::io::{prelude, Read, Write, Cursor};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use zip::{self, read::ZipArchive};

mod ns2_xml_format {
	#[derive(Deserialize, Debug)]
	pub struct PublishedFile {
		pub publishedfileid: u64,
		pub file_url:        String,
		pub time_updated:    u64,
	}

	#[derive(Deserialize, Debug)]
	pub struct PublishedFileDetails {
		pub publishedfile: PublishedFile
	}

	#[derive(Deserialize, Debug)]
	pub struct Response {
		pub publishedfiledetails: PublishedFileDetails
	}

	#[derive(Deserialize, Debug)]
	pub struct Root {
		pub response: Response
	}
}

use self::ns2_xml_format::Root as NS2XMLFormat;

fn download(url: &str) -> Vec<u8> {
	let mut buf = Vec::new();
	let mut easy = ::curl::easy::Easy::new();
	easy.url(url).unwrap();
	{
		let mut transfer = easy.transfer();
		transfer.write_function(|data| {
			buf.extend_from_slice(data);
			Ok(data.len())
		}).unwrap();
		transfer.perform().unwrap();
	}
	buf
}

pub fn update_dependency(dep: &str) {
	let mut dep_path = PathBuf::from("dependencies");
	dep_path.push(dep);
	let dep_path = dep_path.as_path();

	let modid_file = dep_path.join(".modid");
	if modid_file.exists() {
		let mut s = String::new();
		File::open(modid_file).unwrap().read_to_string(&mut s);
		assert_eq!(dep, &s, "What have you done?");

		let format: NS2XMLFormat = serde_xml_rs::deserialize(&*download(&format!(
			"http://mods.ns2cdt.com/ISteamRemoteStorage/GetPublishedFileDetails/V0001?format=xml&publishedfileid={}",
			u64::from_str(dep).unwrap() // we have to convert to decimal
		))).unwrap();

		let path = dep_path.join(".update_timestamp");
		let local_update = if path.exists() {
			File::open(&path).unwrap().read_u64::<LE>().unwrap()
		} else {
			0
		};
		let remote_update = format.response.publishedfiledetails.publishedfile.time_updated;
		if local_update < remote_update {
			File::create(path).unwrap().write_u64::<LE>(remote_update);
		} else {
			let mut archive = ZipArchive::new(Cursor::new(download(&format.response.publishedfiledetails.publishedfile.file_url))).unwrap();
			for i in 0..archive.len() {
				let mut file = archive.by_index(i).unwrap();
				let path = dep_path.join(file.name());
				fs::create_dir_all(path.parent().unwrap()).unwrap();
				let mut buf = Vec::new();
				file.read_to_end(&mut buf).unwrap();
				File::create(path).unwrap().write_all(&buf).unwrap();
			};
		};
	} else {
	};
}

pub fn main<'a>(repo: Repository, matches: &ArgMatches<'a>) {
	fs::create_dir_all("dependencies").unwrap();
	match matches.subcommand() {
		("add", m)   => {
			let url = m.unwrap().value_of("ID").unwrap();
			fs::create_dir(format!("dependencies/{}", url)).unwrap();
			match u64::from_str_radix(url, 16) {
				Ok(modid) => {
					File::create(format!("dependencies/{}/.modid", url)).unwrap().write_all(url.as_bytes()).unwrap();
				},
				Err(_) => {
					let mut submodule = repo.submodule(url, Path::new(&format!("dependencies/{}", url)), true).unwrap();
					submodule.open().unwrap();
					submodule.add_finalize().unwrap();
				}
			};
			update_dependency(url);
		},
		("rm", _m)        => {
			panic!("NYI")
		},
		_ => {
			eprintln!("Not a valid command!");
			exit(1)
		}
	};
}
