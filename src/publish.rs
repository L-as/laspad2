use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write, Cursor};
use std::result::Result as StdResult;
use failure::*;
use zip;
use git2::Repository;
use toml;

use steam::{Error as SteamError, self};
use update;
use compile;
use md_to_bb;
use logger::*;

#[derive(Debug, Fail)]
pub enum PublishError {
	#[fail(display = "Could not publish dummy file to steam")]
	DummyFile,
	#[fail(display = "Branch {} does not exist", branch)]
	NonexistentBranch {
		branch: String
	},
	#[fail(display = "Could not initialize Steam API")]
	NoSteam,
	#[fail(display = "Could not create SteamRemoteStorage API interface")]
	NoSteamRemoteStorage,
	#[fail(display = "Could not create SteamUtils API interface")]
	NoSteamUtils,
	#[fail(display = "Could not upload zip file to remote storage")]
	CantUploadMod,
	#[fail(display = "Could not upload preview to remote storage")]
	CantUploadPreview,
	#[fail(display = "Could not update zip file used")]
	CantUpdateMod,
}

type Result<T> = ::std::result::Result<T, Error>;

#[derive(Deserialize)]
struct Branch {
	name:            Box<str>,
	tags:            Vec<Box<str>>,
	autodescription: bool,
	description:     Box<str>,
	preview:         Box<str>,
}

pub fn generate_description(item: steam::Item) -> Result<String> {
	let mut s: String = format!(
		"[b]Mod ID: {}[/b]\n\n",
		item
	);

	if Path::new(".git").exists() {
		let repo = Repository::open(".")?;
		if let Ok(origin) = repo.find_remote("origin") {
			let origin = origin.url().unwrap();
			let head   = repo.head()?;
			let oid    = head.peel_to_commit()?.id();
			s.push_str(&format!(
				"[b][url={}]git repository[/url][/b]\ncurrent git commit: {}\n\n",
				origin,
				oid
			));
		};
	};

	if Path::new("dependencies").exists() {
		s.push_str("Mods included: [list]\n");
		for dependency in fs::read_dir("dependencies").expect("Couldn't read dependencies directory") {
			let dependency = dependency?;
			let path       = dependency.path();
			let name       = dependency.file_name().into_string().expect("Invalid UTF-8");
			let (name, url) = if let Ok(modid) = u64::from_str_radix(&name, 16) {
				#[derive(Deserialize)]
				struct ModInfo {
					name: Box<str>
				}

				let mut buf = String::new();
				File::open(path.join(".modinfo")).with_context(|_| format!("Couldn't read .modinfo file for {}", &name))?.read_to_string(&mut buf)?;
				let modinfo: ModInfo = toml::from_str(&buf)?;

				let url = format!("http://steamcommunity.com/sharedfiles/filedetails/?id={}", modid);

				(modinfo.name, url.into_boxed_str())
			} else if path.join(".git").exists() {
				let repo   = Repository::open(path)?;
				let origin = repo.find_remote("origin")?;
				let url    = origin.url().unwrap();

				(name.into_boxed_str(), String::from(url).into_boxed_str())
			} else {
				continue
			};
			s.push_str(&format!(
				"  [*] [url={}]{}[/url]\n",
				url,
				name
			));
		};
		s.push_str("[/list]\n\n");
	};

	Ok(s)
}

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

pub fn main(branch_name: &str, retry: bool, log: &Log) -> Result<()> {
	let mut buf = String::new();
	File::open("laspad.toml")?.read_to_string(&mut buf)?;

	let toml: toml::Value = buf.parse()?;

	let branch: Branch = if let toml::Value::Table(mut t) = toml {
		match t.remove(branch_name) {
			Some(b) => b,
			None    => bail!(PublishError::NonexistentBranch { branch: branch_name.to_owned() }),
		}.try_into().unwrap()
	} else {
		unreachable!()
	};

	ensure!(steam::init().is_ok(), PublishError::NoSteam);
	let mut remote = match steam::RemoteStorage::new() {
		Ok(r) => r,
		Err(_) => bail!(PublishError::NoSteamRemoteStorage),
	};
	let mut utils  = match steam::Utils::new() {
		Ok(r) => r,
		Err(_) => bail!(PublishError::NoSteamUtils),
	};

	let modid_file = PathBuf::from(format!(".modid.{}", branch_name));
	let item = if modid_file.exists() {
		steam::Item(u64::from_str_radix(&fs::read_string(&modid_file).context("Could not read the modid file")?, 16)?)
	} else {
		let item = create_workshop_item(&mut remote, &mut utils)?;
		log!(log, 1; "Created Mod ID: {:X}", item.0);
		fs::write(&modid_file, format!("{:X}", item.0).as_bytes()).context("Could not create modid file, next publish will create a new mod!")?;
		item
	};

	update::main(log)?;

	log!(log, 1; "Zipping up files");
	let zip = Vec::new();
	let zip = {
		use std::cell::RefCell;

		let mut cursor = Cursor::new(zip);
		let mut zip    = RefCell::new(zip::ZipWriter::new(cursor));

		let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

		zip.get_mut().start_file(".modinfo", options)?;
		zip.get_mut().write_all(format!("name = \"{}\"", branch.name).as_bytes()).expect("Could not write to zip archive!");

		compile::iterate_files(&Path::new("."), &mut |path, rel_path| {
			log!(log, 2; "{} < {}", rel_path.display(), path.display());
			let mut zip = zip.borrow_mut();
			zip.start_file(rel_path.to_str().unwrap().clone().chars().map(|c|if cfg!(windows) && c=='\\'{'/'} else {c}).collect::<String>(), options)?;
			zip.write_all(&fs::read(path)?).expect("Could not write to zip archive!");
			Ok(())
		}, &mut |rel_path| {
			log!(log, 2; "--- {} ---", rel_path.display());
			//zip.borrow_mut().add_directory(rel_path.to_str()?, options)?;
			Ok(())
		}, log)?;

		zip.get_mut().finish()?.into_inner()
	};

	log!(log, 1; "Finished preparing preview and description");
	let mut preview = fs::read(&*branch.preview).context("Could not read preview")?;
	if preview.len() == 0 { // Steam craps itself when it has 0 length
		preview.push(0);
	};

	let description = md_to_bb::convert(&fs::read_string(&*branch.description).context("Could not read description")?);

	let description = if branch.autodescription {
		let mut s = generate_description(item)?;
		s.push_str(&description);
		s
	} else {
		description
	};

	log!(log, 1; "Uploading zip");
	if remote.file_write("laspad_mod.zip", &zip).is_err() {
		bail!(PublishError::CantUploadMod);
	};

	log!(log, 1; "Uploading preview");
	if remote.file_write("laspad_preview", &preview).is_err() {
		bail!(PublishError::CantUploadPreview);
	};

	let mut request_update = || {
		log!(log, 1; "Requesting workshop item update");
		let u = remote.update_workshop_file(item);
		if u.title(&branch.name).is_err() {
			elog!(log; "Could not update title");
		};
		if u.tags(&branch.tags.iter().map(|s| &**s).collect::<Vec<_>>()).is_err() {
			elog!(log; "Could not update tags");
		};
		if u.description(&description).is_err() {
			elog!(log; "Could not update description");
		};
		if u.preview("laspad_preview").is_err() {
			elog!(log; "Could not update preview");
		};
		if u.contents("laspad_mod.zip").is_err() {
			bail!(PublishError::CantUpdateMod);
		};
		if Path::new(".git").exists() {
			let repo = Repository::open(".").expect("Could not open git repo!");
			let head = repo.head()?;
			let oid = head.peel_to_commit()?.id();
			if u.change_description(&format!("git commit: {}", oid)).is_err() {
				elog!(log; "Could not update version history");
			};
		};
		let apicall = u.commit();

		let result = utils.get_apicall_result::<steam::UpdateItemResult>(apicall);

		let result = StdResult::<_, _>::from(result.result).and(Ok(result.item));
		if let Ok(item) = result {
			log!(log; "Published mod: {}", item);
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

	steam::deinit();

	Ok(())
}
