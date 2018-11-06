use failure::*;
use git2::Repository;
use std::{
	fs,
	io::Cursor,
	path::{Path, PathBuf},
	result::Result as StdResult,
};

use crate::{
	common,
	config,
	steam::{self, GeneralError as SteamError},
	update,
	package,
};

#[derive(Debug, Fail)]
pub enum PublishError {
	#[fail(display = "Could not publish dummy file to steam")]
	DummyFile,
	#[fail(display = "Branch {} does not exist", branch)]
	NonexistentBranch { branch: String },
	#[fail(display = "Could not upload zip file to remote storage")]
	CantUploadMod,
	#[fail(display = "Could not upload preview to remote storage")]
	CantUploadPreview,
	#[fail(display = "Could not update zip file used")]
	CantUpdateMod,
}

type Result<T> = ::std::result::Result<T, Error>;

fn create_workshop_item(
	remote: &mut steam::RemoteStorage,
	utils: &mut steam::Utils,
) -> Result<steam::Item> {
	ensure!(
		remote.file_write("laspad_mod.zip", &[0 as u8]).is_ok(),
		PublishError::DummyFile
	);

	let apicall =
		remote.publish_workshop_file("laspad_mod.zip", "laspad_mod.zip", "dummy", "dummy", &[]);

	let result = utils.get_apicall_result::<steam::PublishItemResult>(apicall);

	Ok(StdResult::<_, _>::from(result.result).and(Ok(result.item))?)
}

pub fn main(branch_name: &str, retry: bool) -> Result<()> {
	common::find_project()?;

	let config = config::get()?;
	ensure!(
		config.contains(branch_name),
		PublishError::NonexistentBranch {
			branch: branch_name.to_owned(),
		}
	);

	log!(2; "Connecting to steam process");
	let client = steam::Client::new()?;
	log!(2; "Accessing Remote Storage API");
	let mut remote = client.remote_storage()?;
	log!(2; "Accessing Utils API");
	let mut utils = client.utils()?;

	log!(2; "Finding mod id for branch");
	let modid_file = PathBuf::from(format!(".modid.{}", branch_name));
	let item = if modid_file.exists() {
		steam::Item(u64::from_str_radix(
			&fs::read_to_string(&modid_file).context("Could not read the modid file")?,
			16,
		)?)
	} else {
		let item = create_workshop_item(&mut remote, &mut utils)?;
		log!(1; "Created Mod ID: {:X}", item.0);
		fs::write(&modid_file, format!("{:X}", item.0).as_bytes())
			.context("Could not create modid file, next publish will create a new mod!")?;
		item
	};
	log!(2; "Mod ID: {:X}", item.0);

	let branch = config.get(branch_name, item)?.unwrap();

	update::main()?;

	let zip = package::zip(branch_name, Cursor::new(Vec::new()))?.into_inner();

	log!(1; "Uploading zip");
	if remote.file_write("laspad_mod.zip", &zip).is_err() {
		bail!(PublishError::CantUploadMod);
	};

	log!(1; "Uploading preview");
	if remote
		.file_write("laspad_preview", &branch.preview()?)
		.is_err()
	{
		bail!(PublishError::CantUploadPreview);
	};

	let mut request_update = || {
		log!(1; "Requesting workshop item update");
		let u = remote.update_workshop_file(item);
		if u.title(&branch.name()?).is_err() {
			elog!("Could not update title");
		};
		if u.tags(&branch.tags()?.iter().map(|s| &**s).collect::<Vec<_>>())
			.is_err()
		{
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
			if u.change_description(&format!("git commit: {}", oid))
				.is_err()
			{
				elog!("Could not update version history");
			};
		};
		let apicall = u.commit();

		let result = utils.get_apicall_result::<steam::UpdateItemResult>(apicall);

		let result = StdResult::<_, _>::from(result.result).and(Ok(result.item));
		if let Ok(item) = result {
			log!("Published mod: {:X}", item.0);
		};

		Ok(result?)
	};

	if retry {
		while let Err(e) = request_update() {
			match e.downcast::<SteamError>() {
				Ok(e) => {
					if e == SteamError::Busy {
						use std::{thread::sleep, time::Duration};
						sleep(Duration::from_secs(5));
					} else {
						bail!(e);
					}
				},
				Err(e) => bail!(e),
			};
		}
	} else {
		request_update()?;
	};

	Ok(())
}
