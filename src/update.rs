use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use serde_xml_rs;
use std::fs::{self, File};
use std::io::{Read, Write, Cursor};
use std::path::Path;
use zip::read::ZipArchive;
use failure::*;
use curl::easy::Easy;

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

pub fn specific(dep: &str, output: &mut Write) -> Result<()> {
	debug!("Updating {}", dep);

	let modid = match u64::from_str_radix(dep, 16) {
		Ok(modid) => modid,
		Err(_)    => {debug!("{} is not a workshop item", dep); return Ok(())},
	};

	debug!("{} is a workshop item", dep);
	let dep_path = &Path::new("dependencies").join(dep);

	let format: NS2XMLFormat = serde_xml_rs::deserialize(&*download(&format!(
		"http://mods.ns2cdt.com/ISteamRemoteStorage/GetPublishedFileDetails/V0001?format=xml&publishedfileid={}",
		modid
	))?).with_context(|_| format!("Could not deserialize XML from Steam for {}", dep))?;

	let path          = dep_path.join(".update_timestamp");
	let local_update  = if path.exists() {
		File::open(&path)?.read_u64::<LE>()?
	} else {
		0
	};
	let remote_update = format.publishedfiledetails.publishedfile.time_updated;
	if local_update < remote_update {
		let _ = writeln!(output, "Local workshop item {} copy is outdated, {} < {}", dep, local_update, remote_update);
		for entry in fs::read_dir(dep_path)? {
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
		let mut archive = ZipArchive::new(Cursor::new(buf)).with_context(|_| format!("Could not read zip archive for {} @ {}", dep, url))?;
		for i in 0..archive.len() {
			let mut file = archive.by_index(i).with_context(|_| format!("Could not access file in zip archive for {}", dep))?;
			let path = dep_path.join(file.name());
			fs::create_dir_all(path.parent().unwrap())?;
			let mut buf = Vec::new();
			file.read_to_end(&mut buf)?;
			File::create(path)?.write_all(&buf)?;
		};
		File::create(path)?.write_u64::<LE>(remote_update)?;
	} else {
		let _ = writeln!(output, "Local workshop item {} copy is up-to-date", dep);
	};

	Ok(())
}


pub fn main(output: &mut Write) -> Result<()> {
	let dependencies = Path::new("dependencies");
	if dependencies.exists() {
		for dependency in fs::read_dir(dependencies)? {
			specific(&dependency?.file_name().into_string().expect("Invalid UTF-8"), output)?;
		};
	};
	Ok(())
}
