use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use serde_xml_rs;
use std::fs::{self, File};
use std::io::{Read, Write, Cursor};
use std::path::Path;
use zip::read::ZipArchive;
use failure::*;
use curl::easy::Easy;

use steam::Item;
use common;

type Result<T> = ::std::result::Result<T, Error>;

mod ns2_xml_format {
	#[derive(Deserialize, Debug)]
	pub struct PublishedFile {
		pub publishedfileid: u64,
		pub file_url:        Box<str>,
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

fn download(url: &str) -> Result<Vec<u8>> {
	let mut buf = Vec::new();
	let mut easy = Easy::new();
	easy.url(url)?;
	{
		let mut transfer = easy.transfer();
		transfer.write_function(|data| {
			buf.extend_from_slice(data);
			Ok(data.len())
		})?;
		transfer.perform()?;
	}
	Ok(buf)
}

pub fn specific<P: AsRef<Path>>(item: Item, path: P) -> Result<()> {
	let path = path.as_ref();

	let format: NS2XMLFormat = serde_xml_rs::deserialize(&*download(&format!(
		"http://mods.ns2cdt.com/ISteamRemoteStorage/GetPublishedFileDetails/V0001?format=xml&publishedfileid={}",
		item.0
	))?).with_context(|_| format!("Could not deserialize XML from Steam for {}", item))?;

	let local_update = {
		let path = path.join(".update_timestamp");
		if path.exists() {
			File::open(&path)?.read_u64::<LE>()?
		} else {
			0
		}
	};

	let remote_update = format.publishedfiledetails.publishedfile.time_updated;
	if local_update < remote_update {
		log!(log, 1; "Local workshop item {} copy is outdated, {} < {}", item, local_update, remote_update);
		for entry in fs::read_dir(path)? {
			let entry = &entry?.path();
			if entry.file_name().unwrap().to_str().unwrap().chars().next().unwrap() != '.' {
				if entry.is_dir() {
					fs::remove_dir_all(entry)?;
				} else {
					fs::remove_file(entry)?;
				};
			};
		};

		let url = &format.publishedfiledetails.publishedfile.file_url;
		let buf = download(url)?;
		let mut archive = ZipArchive::new(Cursor::new(buf)).with_context(|_| format!("Could not read zip archive for {} @ {}", item, url))?;
		for i in 0..archive.len() {
			let mut file = archive.by_index(i).with_context(|_| format!("Could not access file in zip archive for {}", item))?;
			let path = path.join(file.name());
			fs::create_dir_all(path.parent().unwrap())?;
			let mut buf = Vec::new();
			file.read_to_end(&mut buf)?;
			File::create(path)?.write_all(&buf)?;
		};
		File::create(path.join(".update_timestamp"))?.write_u64::<LE>(remote_update)?;
	} else {
		log!(log, 1; "Local workshop item {} copy is up-to-date", item);
	};

	Ok(())
}


pub fn main() -> Result<()> {
	common::find_project()?;

	let dependencies = Path::new("dependencies");
	if dependencies.exists() {
		for dep in fs::read_dir(dependencies)? {
			let path = &dep?.path();
			if let Ok(modid) = u64::from_str_radix(path.file_name().unwrap().to_str().unwrap(), 16) {
				if let Err(e) = specific(Item(modid), path) {
					elog!(log; "Could not update {}: {}", Item(modid), e);
				};
			};
		};
	};

	log!(log; "Finished updating");
	Ok(())
}
