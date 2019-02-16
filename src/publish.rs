use const_cstr::const_cstr;
use derive_more::{Display, From};
use erroneous::Error as EError;
use futures::Future;
use std::{
	ffi::{CStr, CString},
	fs,
	io::{self, Cursor},
	thread::sleep,
	time::Duration,
};
use steam::RemoteStorage;

use crate::{
	config::{self, Branch},
	item::Item,
	package,
	project::Project,
	util,
};

#[derive(Debug, Display, EError, From)]
pub enum Error {
	#[display(fmt = "Could not generate description")]
	DescriptionError(#[error(source)] config::DescriptionError),
	#[display(fmt = "Could not read preview")]
	Preview(#[error(source)] io::Error),
	#[display(
		fmt = "'.modid.{}' does not have a valid format! It should contain the Mod ID of the branch.",
		branch
	)]
	InvalidModIDFileFormat { branch: String },
	#[display(fmt = "Could not write to '.modid.{}'", branch)]
	WriteModIDFile { branch: String, source: io::Error },
	#[display(fmt = "Could not write mod files to remote storage")]
	WriteFiles(#[error(source)] steam::Error),
	#[display(fmt = "Could not update mod")]
	UpdateMod(#[error(source)] steam::Error),
	#[display(fmt = "{}", _0)]
	ReadError(#[error(defer)] util::ReadError),
	#[display(fmt = "Could not create a new mod")]
	CreateMod(#[error(source)] steam::Error),
	#[display(fmt = "Could not package mod into a Zip archive")]
	PackageError(#[error(source)] package::Error),
	#[display(fmt = "Could not access Steamworks SDK interfaces")]
	Interface,
}

const_cstr! {
	PATH_ZIP = "laspad_mod.zip";
	PATH_PREVIEW = "laspad_preview";
}

const WRITE_ZIP_ERROR_MSG: &str = "Couldn't initiate writing ZIP file to Steam Cloud";
const WRITE_PREVIEW_ERROR_MSG: &str = "Couldn't initiate writing preview to Steam Cloud";

macro_rules! repeat {
	($e:expr) => {
		loop {
			match $e {
				Err(steam::Error::Busy) => {
					sleep(Duration::from_millis(50));
				},
				v => break v,
				}
			}
	};
}

fn create_workshop_item<'a>(remote: &'a steam::RemoteStorage<'a>) -> Result<Item, steam::Error> {
	repeat!(remote
		.file_write(PATH_ZIP.as_cstr(), [0])
		.expect(WRITE_ZIP_ERROR_MSG)
		.wait())?;

	let name = const_cstr!("dummy").as_cstr();
	Ok(repeat!({
		remote
			.publish(
				4920,
				PATH_ZIP.as_cstr(),
				PATH_ZIP.as_cstr(),
				name,
				name,
				&[] as &[&CStr],
			)
			.expect("Couldn't initiate uploading dummy mod to Steam")
			.wait()
	})?
	.into())
}

pub fn publish(project: &Project, branch: &Branch, branch_name: &str) -> Result<(), Error> {
	use self::Error::*;

	let mut steam = steam::STEAM.lock().expect("Couldn't lock Steam mutex");
	let client = steam.new_client();
	let remote = client
		.as_ref()
		.and_then(|c| RemoteStorage::new(c))
		.ok_or(Error::Interface)?;
	let item: Item = match branch.item {
		Some(i) => i,
		None => {
			let path = project.path.join(format!(".modid.{}", branch_name));
			if path.exists() {
				util::read_to_string(path)?
					.parse()
					.map_err(|_| InvalidModIDFileFormat {
						branch: branch_name.into(),
					})?
			} else {
				let item = create_workshop_item(&remote).map_err(CreateMod)?;
				info!("Created new Mod ID");
				fs::write(path, &format!("{:X}", item.0)).map_err(|e| WriteModIDFile {
					branch: branch_name.into(),
					source: e,
				})?;
				item
			}
		},
	};
	info!("Mod ID: {}", item);
	let zip = package::package(project, branch, Cursor::new(Vec::new()))?.into_inner();

	// FIXME `repeat` each separately but at the same time somehow
	repeat!({
		let write_mod = remote
			.file_write(PATH_ZIP.as_cstr(), &zip)
			.expect(WRITE_ZIP_ERROR_MSG);

		let preview = branch.preview(project).map_err(Preview)?;
		// mustn't be empty, so we'll make it an empty PNG
		let preview = if preview.len() == 0 {
			include_bytes!("../assets/empty.png")
		} else {
			&preview[..]
		};

		let write_preview = remote
			.file_write(PATH_PREVIEW.as_cstr(), preview)
			.expect(WRITE_PREVIEW_ERROR_MSG);

		write_mod.join(write_preview).wait()
	})
	.map_err(WriteFiles)?;

	repeat!({
		remote
			.update(*item)
			.title(
				&CString::new(branch.name.as_str())
					.expect("Couldn't generate FFI-compatible string"),
			)
			.unwrap()
			.tags(
				&branch
					.tags
					.iter()
					.map(|s| {
						CString::new(s.as_str()).expect("Couldn't generate FFI-compatible string")
					})
					.collect::<Vec<_>>(),
			)
			.unwrap()
			.description(
				&CString::new(branch.description(project, item)?)
					.expect("Couldn't generate FFI-compatible string"),
			)
			.unwrap()
			.preview(PATH_PREVIEW.as_cstr())
			.expect(WRITE_PREVIEW_ERROR_MSG)
			.file(PATH_ZIP.as_cstr())
			.expect(WRITE_ZIP_ERROR_MSG)
			.change_description(
				&CString::new(
					project
						.head()
						.map_or(String::new(), |id| format!("git commit: {}", id)),
				)
				.expect("Couldn't generate FFI-compatible string"),
			)
			.unwrap()
			.finish()
			.wait()
	})
	.map_err(UpdateMod)?;

	Ok(())
}
