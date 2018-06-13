use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;
use std::result;
use failure::*;
use git2::Repository;
use tempfile::NamedTempFile;

use steam::{GeneralError as SteamError, *, self};
use compile;
use common;
use config;

#[derive(Debug, Fail)]
pub enum PublishError {
	#[fail(display = "Branch {} does not exist", branch)]
	NonexistentBranch {
		branch: String
	},
	#[fail(display = "Could not update the contents of the workshop item")]
	CantUpdateMod,
}

type Result<T> = ::std::result::Result<T, Error>;

pub fn main(branch_name: &str, retry: bool) -> Result<()> {
	common::find_project()?;

	let config = config::get()?;
	ensure!(config.contains(branch_name), PublishError::NonexistentBranch { branch: branch_name.to_owned() });

	log!(2; "Connecting to steam process");
	let client = steam::Client::new()?;
	log!(2; "Accessing UGC API");
	let mut ugc = client.ugc()?;
	log!(2; "Accessing Utils API");
	let utils = client.utils()?;

	log!(2; "Finding mod id for branch");
	let modid_file = PathBuf::from(format!(".modid.{}", branch_name));
	let item = if modid_file.exists() {
		steam::Item(u64::from_str_radix(&fs::read_to_string(&modid_file).context("Could not read the modid file")?, 16)?)
	} else {
		let apicall = ugc.create_item();
		let result: steam::CreateItemResult = utils.get_apicall_result(apicall);
		result::Result::from(result.result)?;
		ensure!(!result.legal_agreement_required, "You need to accept the workshop legal agreement on https://steamcommunity.com/app/4920/workshop/");
		let item = result.item;
		log!(1; "Created Mod ID: {:X}", item.0);
		fs::write(&modid_file, format!("{:X}", item.0).as_bytes()).context("Could not create modid file, next publish will create a new mod!")?;
		item
	};
	log!(2; "Mod ID: {:X}", item.0);

	compile::main()?;

	let branch = config.get(branch_name, item)?.unwrap();

	let mut request_update = || {
		log!(1; "Requesting workshop item update");
		let u = ugc.update_item(AppID(4920), item);
		if u.title(&branch.name()?).is_err() {
			elog!("Could not update title");
		};
		if u.tags(&branch.tags()?.iter().map(|s| &**s).collect::<Vec<_>>()).is_err() {
			elog!("Could not update tags");
		};
		if u.description(&branch.description()?).is_err() {
			elog!("Could not update description");
		};
		let mut preview_file = NamedTempFile::new()?;
		preview_file.write_all(&branch.preview()?)?;
		if u.preview(preview_file.path()).is_err() {
			elog!("Could not update preview");
		};
		if u.content(&Path::new("compiled").canonicalize()?).is_err() {
			bail!(PublishError::CantUpdateMod);
		};
		let update_note = if Path::new(".git").exists() {
			let repo = Repository::open(".").expect("Could not open git repo!");
			let head = repo.head()?;
			let oid = head.peel_to_commit()?.id();
			Some(format!("git commit: {}", oid))
		} else {None};
		let apicall = u.submit(update_note.as_ref().map(|s| s.as_ref()));

		let result: SubmitItemUpdateResult = utils.get_apicall_result(apicall);

		result::Result::from(result.result)?;

		ensure!(!result.legal_agreement_required, "You need to accept the workshop legal agreement on https://steamcommunity.com/app/4920/workshop/");

		let item = result.item;

		log!("Published mod: {:X}", item.0);

		Ok(item)
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
