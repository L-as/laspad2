use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use curl::easy::Easy;
use chrono::{Utc, TimeZone};
use derive_more::{Display, From};
use erroneous::Error as EError;
use std::{
	fs::{self, File},
	io::{self, Cursor, Read, Write},
	path::{Path, PathBuf},
};
use zip::ZipArchive;

use crate::item::Item;

mod xml {
	use serde_derive::Deserialize;

	#[derive(Deserialize, Debug)]
	pub struct PublishedFile {
		pub publishedfileid: u64,
		pub file_url:        Box<str>,
		pub time_updated:    u64,
	}

	#[derive(Deserialize, Debug)]
	pub struct PublishedFileDetails {
		pub publishedfile: PublishedFile,
	}

	#[derive(Deserialize, Debug)]
	pub struct Root {
		pub publishedfiledetails: PublishedFileDetails,
	}
}

fn get(url: &str) -> Result<Vec<u8>, curl::Error> {
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

#[derive(Debug, Display, EError, From)]
pub enum Error {
	#[display(fmt = "Could not download XML specification for mod")]
	XMLLoad(#[error(source)] curl::Error),
	#[display(fmt = "Could not parse XML specification for mod")]
	XMLRead, // Can not contain serde_xml_rs::Error, since it's not Sync
	#[display(fmt = "Could not read .update_timestamp file")]
	TimeStamp(#[error(source)] io::Error),
	#[display(fmt = "Could not create new path {}", "_0.display()")]
	Create(PathBuf, #[error(source)] io::Error),
	#[display(fmt = "Could not create target directory")]
	CreateTarget(#[error(source)] io::Error),
	#[display(fmt = "Could not remove target directory")]
	RemoveTarget(#[error(source)] io::Error),
	#[display(fmt = "Could not download Zip archive")]
	ZipLoad(#[error(source)] curl::Error),
	#[display(fmt = "Could not parse Zip archive")]
	ZipRead(#[error(source)] zip::result::ZipError),
	#[display(fmt = "Could not read file {} in Zip archive", _0)]
	ZipReadFile(String, #[error(source)] io::Error),
}

pub fn download(item: Item, path: impl AsRef<Path>) -> Result<(), Error> {
	let path = path.as_ref();

	let format: xml::Root = serde_xml_rs::from_reader(&*get(&format!(
		"http://mods.ns2cdt.com/ISteamRemoteStorage/GetPublishedFileDetails/V0001?format=xml&publishedfileid={}",
		item.0
	)).map_err(Error::XMLLoad)?).map_err(|_| Error::XMLRead)?;

	let local_update = {
		let path = path.join(".update_timestamp");
		if path.exists() {
			File::open(&path)
				.and_then(|mut f| f.read_u64::<LE>())
				.map_err(Error::TimeStamp)?
		} else {
			0
		}
	};

	let remote_update = format.publishedfiledetails.publishedfile.time_updated;
	if local_update < remote_update {
		if local_update > 0 {
			info!(
				"Workshop item {} is outdated, old: {}, new: {}",
				item, Utc.timestamp(local_update as i64, 0).date(), Utc.timestamp(remote_update as i64, 0).date()
			);
		} else {
			info!(
				"Workshop item {} is outdated, old: None, new: {}",
				item, Utc.timestamp(remote_update as i64, 0).date()
			);
		}
		if path.exists() {
			fs::remove_dir_all(&path).map_err(Error::RemoveTarget)?;
		} else {
			fs::create_dir_all(&path).map_err(Error::CreateTarget)?;
		}

		let url = &format.publishedfiledetails.publishedfile.file_url;
		let buf = get(url).map_err(Error::ZipLoad)?;
		let mut archive = ZipArchive::new(Cursor::new(buf))?;
		for i in 0..archive.len() {
			let mut file = archive.by_index(i)?;
			let file_path = path.join(file.name());
			let file_parent = file_path.parent().expect("Could not get parent of path");
			fs::create_dir_all(file_parent).map_err(|e| Error::Create(file_parent.into(), e))?;
			let mut buf = Vec::new();
			file.read_to_end(&mut buf)
				.map_err(|e| Error::ZipReadFile(file.name().into(), e))?;
			File::create(&file_path)
				.and_then(|mut f| f.write_all(&buf))
				.map_err(|e| Error::Create(file_path.into(), e))?;
		}
		File::create(path.join(".update_timestamp"))
			.and_then(|mut f| f.write_u64::<LE>(remote_update))
			.map_err(|e| Error::Create(path.join(".update_timestamp"), e))?;
	} else {
		debug!("Local workshop item {:8X} copy is up-to-date", item.0);
	};

	Ok(())
}
