#![allow(dead_code)]

use std::ffi::CString;
use std::mem::{forget, size_of, transmute, zeroed};
use std::error;
use std::fmt;

include!("steam_ffi.rs");

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item(pub u64);

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct APICall(u64);

#[repr(i64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum APICallFailureReason {
	None              = -1,
	SteamGone         = 0,
	NetworkFailure    = 1,
	InvalidHandle     = 2,
	MismatchedAPICall = 3,
}

pub trait APICallResult {
	const CALLBACK_ID: u32;
}

#[repr(packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublishItemResult {
	pub result:           SteamResult,
	pub item:             Item,
	    accept_agreement: bool,
}

impl APICallResult for PublishItemResult {
	const CALLBACK_ID: u32 = 1309;
}

#[repr(packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateItemResult {
	pub result:           SteamResult,
	pub item:             Item,
	    accept_agreement: bool,
}

impl APICallResult for UpdateItemResult {
	const CALLBACK_ID: u32 = 1316;
}

pub struct ItemUpdater<'a> {
	handle: UpdateHandle,
	storage: &'a RemoteStorage
}

impl<'a> ItemUpdater<'a> {
	pub fn tags(&self, tags: &[&str]) -> Result<&Self, ()> {
		let tags: Vec<*const i8> = tags.iter().map(|&s| {
			let s = CString::new(s).unwrap();
			let ptr = s.as_ptr() as *const i8;
			forget(s); // yeah....
			ptr
		}).collect();
		let tags = Strings {
			elements: tags.as_slice().as_ptr() as *const *const i8,
			length: tags.len() as i32,
		};

		if unsafe { SteamAPI_ISteamRemoteStorage_UpdatePublishedFileTags(self.storage.0, self.handle, &tags as *const Strings) } {
			Ok(&self)
		} else {
			Err(())
		}
	}
	pub fn contents(&self, contents_path: &str) -> Result<&Self, ()> {
		if unsafe { SteamAPI_ISteamRemoteStorage_UpdatePublishedFileFile(self.storage.0, self.handle, CString::new(contents_path).unwrap().as_ptr()) } {
			Ok(&self)
		} else {
			Err(())
		}
	}
	pub fn preview(&self, preview_path: &str) -> Result<&Self, ()> {
		if unsafe { SteamAPI_ISteamRemoteStorage_UpdatePublishedFilePreviewFile(self.storage.0, self.handle, CString::new(preview_path).unwrap().as_ptr()) } {
			Ok(&self)
		} else {
			Err(())
		}
	}
	pub fn description(&self, description: &str) -> Result<&Self, ()> {
		if unsafe { SteamAPI_ISteamRemoteStorage_UpdatePublishedFileDescription(self.storage.0, self.handle, CString::new(description).unwrap().as_ptr()) } {
			Ok(&self)
		} else {
			Err(())
		}
	}
	pub fn title(&self, title: &str) -> Result<&Self, ()> {
		if unsafe { SteamAPI_ISteamRemoteStorage_UpdatePublishedFileTitle(self.storage.0, self.handle, CString::new(title).unwrap().as_ptr()) } {
			Ok(&self)
		} else {
			Err(())
		}
	}
	pub fn change_description(&self, change_description: &str) -> Result<&Self, ()> {
		if unsafe { SteamAPI_ISteamRemoteStorage_UpdatePublishedFileSetChangeDescription(self.storage.0, self.handle, CString::new(change_description).unwrap().as_ptr()) } {
			Ok(&self)
		} else {
			Err(())
		}
	}
	pub fn commit(&self) -> APICall {
		unsafe { SteamAPI_ISteamRemoteStorage_CommitPublishedFileUpdate(self.storage.0, self.handle) }
	}
}

pub struct RemoteStorage(*mut RemoteStorageImpl);

impl RemoteStorage {
	pub fn new() -> Result<Self, ()> {
		let ptr = unsafe {SteamRemoteStorage()};
		if ptr.is_null() {
			Err(())
		} else {
			Ok(RemoteStorage(ptr))
		}
	}

	pub fn file_write(&mut self, name: &str, data: &[u8]) -> Result<(), ()> {
		if unsafe {SteamAPI_ISteamRemoteStorage_FileWrite(
			self.0,
			CString::new(name).unwrap().as_ptr(),
			data.as_ptr(),
			data.len() as u32
		)} {
			Ok(())
		} else {
			Err(())
		}
	}

	pub fn publish_workshop_file(&mut self, contents_path: &str, preview_path: &str, title: &str, description: &str, tags: &[&str]) -> APICall {
		let tags = Strings {
			elements: tags.iter().map(|&s| CString::new(s).unwrap().as_ptr()).collect::<Vec<_>>().as_ptr(),
			length: tags.len() as i32,
		};

		unsafe {SteamAPI_ISteamRemoteStorage_PublishWorkshopFile(
			self.0,
			CString::new(contents_path).unwrap().as_ptr(),
			CString::new(preview_path).unwrap().as_ptr(),
			4920,
			CString::new(title).unwrap().as_ptr(),
			CString::new(description).unwrap().as_ptr(),
			Visibility::Public,
			&tags as *const Strings,
			FileType::Community
		)}
	}

	pub fn update_workshop_file<'a>(&'a mut self, item: Item) -> ItemUpdater<'a> {
		ItemUpdater {
			handle: unsafe { SteamAPI_ISteamRemoteStorage_CreatePublishedFileUpdateRequest(self.0, item) },
			storage: self,
		}
	}
}

pub struct Utils(*mut UtilsImpl);

impl Utils {
	pub fn new() -> Result<Self, ()> {
		let ptr = unsafe {SteamUtils()};
		if ptr.is_null() {
			Err(())
		} else {
			Ok(Utils(ptr))
		}
	}

	pub fn is_apicall_completed(&self, call: APICall) -> bool {
		let mut b = false;
		unsafe { SteamAPI_ISteamUtils_IsAPICallCompleted(self.0, call, &mut b as *mut bool) }
	}

	pub fn get_apicall_result<T: APICallResult + fmt::Debug>(&self, call: APICall) -> T {
		while !self.is_apicall_completed(call) {
			use std::{thread, time::Duration};
			thread::sleep(Duration::from_millis(200));
		};

		let mut result: T = unsafe { zeroed() };

		let mut _b = false; // ignore Steam saying we have errors, because we don't. Steam just has trouble accepting that fact.
		assert!(unsafe {SteamAPI_ISteamUtils_GetAPICallResult(
			self.0,
			call,
			transmute(&mut result),
			size_of::<T>() as u32,
			T::CALLBACK_ID,
			&mut _b as *mut bool
		)});

		result
	}
}

pub fn init() -> Result<(), ()> {
	if unsafe { SteamAPI_Init() } {
		Ok(())
	} else {
		Err(())
	}
}

pub fn deinit() {
	unsafe { SteamAPI_Shutdown() }
}
