use byteorder::{LE, ReadBytesExt, WriteBytesExt};

use serde_xml_rs;

use std::fs::{self, File};
use std::io::{Result, Read, Write, Cursor};
use std::path::Path;

use zip::read::ZipArchive;

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

pub fn specific(dep: &str) -> Result<()> {
	debug!("Updating {}", dep);

	let modid = u64::from_str_radix(dep, 16);
	if modid.is_err() {
		debug!("{} is not a workshop item", dep);
		return Ok(());
	};
	let modid = modid.unwrap();

	debug!("{} is a workshop item", dep);
	let dep_path = Path::new("dependencies").join(dep);

	let format: NS2XMLFormat = serde_xml_rs::deserialize(&*download(&format!(
		"http://mods.ns2cdt.com/ISteamRemoteStorage/GetPublishedFileDetails/V0001?format=xml&publishedfileid={}",
		modid
	))).unwrap();

	let path          = dep_path.join(".update_timestamp");
	let local_update  = if path.exists() {
		File::open(&path)?.read_u64::<LE>()?
	} else {
		0
	};
	let remote_update = format.publishedfiledetails.publishedfile.time_updated;
	if local_update < remote_update {
		println!("Local workshop item {} copy is outdated, {} < {}", dep, local_update, remote_update);
		File::create(path)?.write_u64::<LE>(remote_update)?;
		let mut archive = ZipArchive::new(Cursor::new(download(&format.publishedfiledetails.publishedfile.file_url))).unwrap();
		for i in 0..archive.len() {
			let mut file = archive.by_index(i).unwrap();
			let path = dep_path.join(file.name());
			fs::create_dir_all(path.parent().unwrap())?;
			let mut buf = Vec::new();
			file.read_to_end(&mut buf)?;
			File::create(path)?.write_all(&buf)?;
		};
	} else {
		println!("Local workshop item {} copy is up-to-date", dep);
	};

	Ok(())
}


pub fn main() -> Result<()> {
	let dependencies = Path::new("dependencies");
	if dependencies.exists() {
		for dependency in fs::read_dir(dependencies)? {
			specific(&dependency?.file_name().into_string().unwrap())?;
		};
	};
	Ok(())
}