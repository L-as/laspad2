use git2::Repository;

use clap::ArgMatches;

use byteorder::{LE, ReadBytesExt, WriteBytesExt};

use serde_xml_rs;

use std::fs::{self, File};
use std::process::exit;
use std::io::{Read, Write, Cursor};
use std::path::PathBuf;

use zip::read::ZipArchive;

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

use self::ns2_xml_format::Response as NS2XMLFormat;

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
	trace!("Update {}", dep);

	let mut dep_path = PathBuf::from("dependencies");
	dep_path.push(dep);
	let dep_path = dep_path.as_path();

	match u64::from_str_radix(dep, 16) {
		Ok(modid) => { // workshop dep
			trace!("{} is workshop item", dep);
			let format: NS2XMLFormat = serde_xml_rs::deserialize(&*download(&format!(
				"http://mods.ns2cdt.com/ISteamRemoteStorage/GetPublishedFileDetails/V0001?format=xml&publishedfileid={}",
				modid
			))).unwrap();

			let path = dep_path.join(".update_timestamp");
			let local_update = if path.exists() {
				File::open(&path).unwrap().read_u64::<LE>().unwrap()
			} else {
				0
			};
			let remote_update = format.publishedfiledetails.publishedfile.time_updated;
			if local_update < remote_update {
				info!("Local workshop item copy is outdated, {} < {}", local_update, remote_update);
				File::create(path).unwrap().write_u64::<LE>(remote_update).unwrap();
				let mut archive = ZipArchive::new(Cursor::new(download(&format.publishedfiledetails.publishedfile.file_url))).unwrap();
				for i in 0..archive.len() {
					let mut file = archive.by_index(i).unwrap();
					let path = dep_path.join(file.name());
					fs::create_dir_all(path.parent().unwrap()).unwrap();
					let mut buf = Vec::new();
					file.read_to_end(&mut buf).unwrap();
					File::create(path).unwrap().write_all(&buf).unwrap();
				};
			} else {
				info!("Local workshop item copy is up-to-date");
			};
		},
		Err(_) => { // git repo
			trace!("{} is git submodule", dep);
		},
	};
}

pub fn main<'a>(repo: Repository, matches: &ArgMatches<'a>) {
	fn create_dep_dir(dep: &str) -> PathBuf {
		trace!("Making dependency directory {}", dep);

		let path = PathBuf::from(&format!("dependencies/{}", dep));
		if path.exists() {
			error!("Dependency already exists!");
			exit(1);
		};

		fs::create_dir(&path).unwrap();
		path
	}

	fs::create_dir_all("dependencies").unwrap();
	match matches.subcommand() {
		("add", m)   => {
			let url = m.unwrap().value_of("ID").unwrap();

			trace!("add {}", url);
			match u64::from_str_radix(url, 16) {
				Ok(_) => {
					trace!("Workshop item dependency");
					let dep = url.to_uppercase();
					create_dep_dir(&dep);
					update_dependency(&dep);
				},
				Err(_) => {
					trace!("git submodule dependency");
					let path = create_dep_dir(url);
					let mut submodule = repo.submodule(url, &path, true).unwrap();
					submodule.open().unwrap();
					submodule.add_finalize().unwrap();
					update_dependency(url);
				}
			};
		},
		("rm", _m)        => {
			panic!("NYI")
		},
		_ => {
			error!("Not a valid command!");
			exit(1)
		}
	};
}
