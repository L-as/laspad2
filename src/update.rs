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
	easy.url(url).unwrap_or_else(|e| panic!("Could not set URL '{}': {:?}", url, e));
	{
		let mut transfer = easy.transfer();
		transfer.write_function(|data| {
			buf.extend_from_slice(data);
			Ok(data.len())
		}).unwrap_or_else(|e| panic!("Could not set write function for URL '{}': {:?}", url, e));
		transfer.perform().unwrap_or_else(|e| panic!("Could not perform transfer for URL '{}': {:?}", url, e));
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
	))).unwrap_or_else(|e| panic!("Could not deserialize XML for {}: {:?}", dep, e));

	let path          = dep_path.join(".update_timestamp");
	let local_update  = if path.exists() {
		File::open(&path)?.read_u64::<LE>()?
	} else {
		0
	};
	let remote_update = format.publishedfiledetails.publishedfile.time_updated;
	if local_update < remote_update {
		println!("Local workshop item {} copy is outdated, {} < {}", dep, local_update, remote_update);
		let url = &format.publishedfiledetails.publishedfile.file_url;
		let buf = download(url);
		if cfg!(target_os = "windows") {
			use std::process::Command;
			use std::env::current_exe;

			let path = dep_path.join(".mod.zip");
			File::create(&path).unwrap().write_all(&buf).unwrap();
			let status = Command::new("cscript")
				.arg("//B")
				.arg(current_exe().unwrap().join("unzip.vbs"))
				.arg(&path)
				.arg(dep_path)
				.status()
				.unwrap();

			if !status.success() {
				panic!("Could not read zip archive for {} @ {}: {}", dep, url, status);
			};
		} else {
			let mut archive = ZipArchive::new(Cursor::new(buf)).unwrap_or_else(|e| panic!("Could not read zip archive for {} @ {}: {:?}", dep, url, e));
			for i in 0..archive.len() {
				let mut file = archive.by_index(i).unwrap_or_else(|e| panic!("Could not access file in zip archive for {}: {:?}", dep, e));
				let path = dep_path.join(file.name());
				fs::create_dir_all(path.parent().unwrap())?;
				let mut buf = Vec::new();
				file.read_to_end(&mut buf)?;
				File::create(path)?.write_all(&buf)?;
			};
		}
		File::create(path)?.write_u64::<LE>(remote_update)?;
	} else {
		println!("Local workshop item {} copy is up-to-date", dep);
	};

	Ok(())
}


pub fn main() -> Result<()> {
	let dependencies = Path::new("dependencies");
	if dependencies.exists() {
		for dependency in fs::read_dir(dependencies)? {
			specific(&dependency?.file_name().into_string().unwrap_or_else(|e| panic!("Could not access name for dependency: {:?}", e)))?;
		};
	};
	Ok(())
}
