use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Write, Cursor};
use std::result::Result as StdResult;
use failure::*;
use zip;
use git2::Repository;

use steam::{GeneralError as SteamError, self};
use update;
use compile;
use common;
use config;

#[derive(Debug, Fail)]
pub enum PublishError {
	#[fail(display = "Could not publish dummy file to steam")]
	DummyFile,
	#[fail(display = "Branch {} does not exist", branch)]
	NonexistentBranch {
		branch: String
	},
	#[fail(display = "Could not upload zip file to remote storage")]
	CantUploadMod,
	#[fail(display = "Could not upload preview to remote storage")]
	CantUploadPreview,
	#[fail(display = "Could not update zip file used")]
	CantUpdateMod,
}

type Result<T> = ::std::result::Result<T, Error>;

fn create_workshop_item(remote: &mut steam::RemoteStorage, utils: &mut steam::Utils) -> Result<steam::Item> {
	ensure!(remote.file_write("laspad_mod.zip", &[0 as u8]).is_ok(), PublishError::DummyFile);

	let apicall = remote.publish_workshop_file(
		"laspad_mod.zip",
		"laspad_mod.zip",
		"dummy",
		"dummy",
		&[]
	);

	let result = utils.get_apicall_result::<steam::PublishItemResult>(apicall);

	Ok(StdResult::<_, _>::from(result.result).and(Ok(result.item))?)
}

pub fn main(branch_name: &str, retry: bool) -> Result<()> {
	common::find_project()?;

	let config = config::get()?;
	ensure!(config.contains(branch_name), PublishError::NonexistentBranch { branch: branch_name.to_owned() });

	let client     = steam::Client::new()?;
	let mut remote = client.remote_storage()?;
	let mut utils  = client.utils()?;

	let modid_file = PathBuf::from(format!(".modid.{}", branch_name));
	let item = if modid_file.exists() {
		steam::Item(u64::from_str_radix(&fs::read_to_string(&modid_file).context("Could not read the modid file")?, 16)?)
	} else {
		let item = create_workshop_item(&mut remote, &mut utils)?;
		log!(1; "Created Mod ID: {:X}", item.0);
		fs::write(&modid_file, format!("{:X}", item.0).as_bytes()).context("Could not create modid file, next publish will create a new mod!")?;
		item
	};

	let branch = config.get(branch_name, item)?.unwrap();

	update::main()?;

	log!(1; "Zipping up files");
	let zip = Vec::new();
	let zip = {
		use walkdir::WalkDir;

		let mut cursor = Cursor::new(zip);
		let mut zip    = zip::ZipWriter::new(cursor);

		let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

		zip.start_file(".modinfo", options)?;
		zip.write_all(format!("name = \"{}\"", branch.name()?).as_bytes()).expect("Could not write to zip archive!");

		compile::main()?;
		for entry in WalkDir::new("compiled").follow_links(true) {
			let entry = entry?;
			let entry = entry.path(); // I have no idea why I have to do this in two statements
			if entry.is_file() {
				let rel = entry.strip_prefix("compiled")?;
				log!(2; "{} < {}", rel.display(), entry.display());
				zip.start_file(rel.to_str().unwrap().clone().chars().map(|c|if cfg!(windows) && c=='\\'{'/'} else {c}).collect::<String>(), options)?;
				zip.write_all(&fs::read(entry)?).expect("Could not write to zip archive!");
			}
		};

		zip.finish()?.into_inner()
	};

	log!(1; "Uploading zip");
	if remote.file_write("laspad_mod.zip", &zip).is_err() {
		bail!(PublishError::CantUploadMod);
	};

	log!(1; "Uploading preview");
	if remote.file_write("laspad_preview", &branch.preview()?).is_err() {
		bail!(PublishError::CantUploadPreview);
	};

	let mut request_update = || {
		log!(1; "Requesting workshop item update");
		let u = remote.update_workshop_file(item);
		if u.title(&branch.name()?).is_err() {
			elog!("Could not update title");
		};
		if u.tags(&branch.tags()?.iter().map(|s| &**s).collect::<Vec<_>>()).is_err() {
			elog!("Could not update tags");
		};
		if u.description(&branch.description()?).is_err() {
			elog!("Could not update description");
		};
		if u.preview("laspad_preview").is_err() {
			elog!("Could not update preview");
		};
		if u.contents("laspad_mod.zip").is_err() {
			bail!(PublishError::CantUpdateMod);
		};
		if Path::new(".git").exists() {
			let repo = Repository::open(".").expect("Could not open git repo!");
			let head = repo.head()?;
			let oid = head.peel_to_commit()?.id();
			if u.change_description(&format!("git commit: {}", oid)).is_err() {
				elog!("Could not update version history");
			};
		};
		let apicall = u.commit();

		let result = utils.get_apicall_result::<steam::UpdateItemResult>(apicall);

		let result = StdResult::<_, _>::from(result.result).and(Ok(result.item));
		if let Ok(item) = result {
			log!("Published mod: {}", item);
		};

		Ok(result?)
	};

	if retry {
		while let Err(e) = request_update() {
			match e.downcast::<SteamError>() {
				Ok(e) => if e == SteamError::Busy {
					use std::{thread::sleep, time::Duration};
					sleep(Duration::from_secs(5));
				} else {
					bail!(e);
				},
				Err(e) => bail!(e),
			};
		};
	} else {
		request_update()?;
	};

	Ok(())
}
