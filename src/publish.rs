use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write, Cursor};
use std::process::exit;

use steam::{SteamResult, self};
use update;
use compile;
use md_to_bb;

use zip;

use git2::Repository;

use toml;

#[derive(Deserialize)]
struct Branch {
	name:            Box<str>,
	tags:            Vec<Box<str>>,
	autodescription: bool,
	description:     Box<str>,
	preview:         Box<str>,
}

pub fn generate_description(modid: u64) -> String {
	let mut s: String = format!(
		"[b]Mod ID: {:X}[/b]\n\n",
		modid
	);

	if Path::new(".git").exists() {
		let repo = Repository::open(".").unwrap();
		if let Ok(origin) = repo.find_remote("origin") {
			let origin = origin.url().unwrap();
			let head   = repo.head().unwrap();
			let oid    = head.peel_to_commit().unwrap().id();
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
			let dependency = dependency.unwrap();
			let path       = dependency.path();
			let name       = dependency.file_name().into_string().unwrap();
			let (name, url) = if let Ok(modid) = u64::from_str_radix(&name, 16) {
				#[derive(Deserialize)]
				struct ModInfo {
					name: Box<str>
				}

				let mut buf = String::new();
				File::open(path.join(".modinfo")).expect("Couldn't read .modinfo file!").read_to_string(&mut buf).unwrap();
				let modinfo: ModInfo = toml::from_str(&buf).unwrap();

				let url = format!("http://steamcommunity.com/sharedfiles/filedetails/?id={}", modid);

				(modinfo.name, url.into_boxed_str())
			} else if path.join(".git").exists() {
				let repo   = Repository::open(path).unwrap();
				let origin = repo.find_remote("origin").unwrap();
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

	s
}

fn create_workshop_item(remote: &mut steam::RemoteStorage, utils: &mut steam::Utils) -> steam::Item {
	remote.file_write("laspad_mod.zip", &[0 as u8]).unwrap_or_else(|_| {
		error!("Could not upload dummy file");
		exit(1)
	});

	let apicall = remote.publish_workshop_file(
		"laspad_mod.zip",
		"laspad_mod.zip",
		"dummy",
		"dummy",
		&[]
	);

	let result = utils.get_apicall_result::<steam::PublishItemResult>(apicall);

	if result.result == SteamResult::OK {
		result.item
	} else {
		error!("Could not publish mod: {:?}", result.result);
		exit(1)
	}
}

pub fn main(branch_name: &str) {
	let mut buf = String::new();
	File::open("laspad.toml").unwrap().read_to_string(&mut buf).unwrap();

	let toml: toml::Value = buf.parse().unwrap();

	let branch: Branch = if let toml::Value::Table(mut t) = toml {
		t.remove(branch_name).unwrap_or_else(|| {
			error!("Branch {} does not exist!", branch_name);
			exit(1)
		}).try_into().unwrap_or_else(|e| {
			error!("Could not deserialize laspad.toml: {}", e);
			exit(1)
		})
	} else {
		unreachable!()
	};

	steam::init().unwrap_or_else(|_| {
		error!("laspad could not initialize Steam API");
		exit(1)
	});
	let mut remote = steam::RemoteStorage::new().unwrap_or_else(|_| {
		error!("Could not create SteamRemoteStorage");
		exit(1)
	});
	let mut utils  = steam::Utils::new().unwrap_or_else(|_| {
		error!("Could not create SteamUtils");
		exit(1)
	});

	let modid_file = PathBuf::from(format!(".modid.{}", branch_name));
	let item = if modid_file.exists() {
		let mut buf = String::new();
		File::open(&modid_file).and_then(|mut f| f.read_to_string(&mut buf)).unwrap_or_else(|e| {
			error!("Could not read {:?}: {}", modid_file, e);
			exit(1)
		});
		steam::Item(u64::from_str_radix(&buf, 16).unwrap())
	} else {
		let item = create_workshop_item(&mut remote, &mut utils);
		println!("Created Mod ID: {:X}", item.0);
		File::create(&modid_file).and_then(|mut f| f.write_all(format!("{:X}", item.0).as_bytes())).unwrap_or_else(|_| {
			error!("Could not write {:X} to {:?}", item.0, modid_file);
		});
		item
	};

	update::main().unwrap();

	println!("Zipping up files");
	let zip = Vec::new();
	let zip = {
		use std::cell::RefCell;

		let mut cursor = Cursor::new(zip);
		let mut zip    = RefCell::new(zip::ZipWriter::new(cursor));

		let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

		zip.get_mut().start_file(".modinfo", options).unwrap();
		zip.get_mut().write_all(format!("name = \"{}\"", branch.name).as_bytes()).expect("Could not write to zip archive!");

		compile::iterate_files(&Path::new("."), &mut |path, rel_path| {
			trace!("{:?} < {:?}", rel_path, path);
			let mut zip = zip.borrow_mut();
			zip.start_file(rel_path.to_str().unwrap(), options).unwrap();
			let mut buf = Vec::new();
			File::open(path).expect("Could not open file!").read_to_end(&mut buf).expect("Could not read file!");
			zip.write_all(&buf).expect("Could not write to zip archive!");
			Ok(())
		}, &mut |rel_path| {
			trace!("--- {:?} ---", rel_path);
			//zip.borrow_mut().add_directory(rel_path.to_str().unwrap(), options).unwrap();
			Ok(())
		}).unwrap();

		zip.get_mut().finish().unwrap().into_inner()
	};

	println!("Finished preparing preview and description");
	let mut preview = Vec::new();
	File::open(&*branch.preview).and_then(|mut f| f.read_to_end(&mut preview)).unwrap_or_else(|e| {
		error!("Could not read preview: {}", e);
		exit(1);
	});
	if preview.len() == 0 { // Steam craps itself when it has 0 length
		preview.push(0);
	};

	let mut description = String::new();
	File::open(&*branch.description).and_then(|mut f| f.read_to_string(&mut description)).unwrap_or_else(|e| {
		error!("Could not read description: {}", e);
		exit(1)
	});

	let description = md_to_bb::convert(&description);

	let description = if branch.autodescription {
		let mut s = generate_description(item.0);
		s.push_str(&description);
		s
	} else {
		description
	};

	println!("Uploading zip");
	if remote.file_write("laspad_mod.zip", &zip).is_err() {
		error!("Could not write mod file to steam!");
		exit(1)
	};

	println!("Uploading preview");
	if remote.file_write("laspad_preview", &preview).is_err() {
		error!("Could not write preview file to steam!");
		exit(1)
	};

	println!("Requesting workshop item update");
	let u = remote.update_workshop_file(item);
	if u.title(&branch.name).is_err() {
		error!("Could not update title");
	};
	if u.tags(&branch.tags.iter().map(|s| &**s).collect::<Vec<_>>()).is_err() {
		error!("Could not update tags");
	};
	if u.description(&description).is_err() {
		error!("Could not update description");
	};
	if u.preview("laspad_preview").is_err() {
		error!("Could not update preview");
	};
	if u.contents("laspad_mod.zip").is_err() {
		error!("Could not update zip");
	};
	if Path::new(".git").exists() {
		let repo = Repository::open(".").expect("Could not open git repo!");
		let head = repo.head().unwrap();
		let oid = head.peel_to_commit().unwrap().id();
		if u.change_description(&format!("git commit: {}", oid)).is_err() {
			error!("Could not update version history");
		};
	};
	let apicall = u.commit();

	let result = utils.get_apicall_result::<steam::UpdateItemResult>(apicall);

	if result.result == SteamResult::OK {
		println!("Published mod: {:X}", result.item.0);
	} else {
		error!("Could not publish mod: {:?}", result.result);
		exit(1)
	};
}
