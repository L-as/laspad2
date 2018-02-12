#![allow(dead_code)]

use std::ffi::CString;
use std::mem::{size_of, transmute, zeroed};
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
pub struct PublishItemResult {
	pub result:           SteamResult,
	pub item:             Item,
	    accept_agreement: bool,
}

impl APICallResult for PublishItemResult {
	const CALLBACK_ID: u32 = 1309;
}

#[repr(packed)]
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
		let tags = Strings {
			elements: tags.iter().map(|&s| CString::new(s).unwrap().as_ptr()).collect::<Vec<_>>().as_ptr(),
			length: tags.len() as u32,
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
			length: tags.len() as u32,
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
	pub fn is_apicall_completed(&self, call: APICall) -> bool {
		let mut b = false;
		unsafe { SteamAPI_ISteamUtils_IsAPICallCompleted(self.0, call, &mut b as *mut bool) }
	}

	pub fn get_apicall_result<T: APICallResult>(&self, call: APICall) -> Result<T, APICallFailureReason> {
		while !self.is_apicall_completed(call) {};

		let mut result: T = unsafe { zeroed() };

		let mut b = false;
		assert!(unsafe {SteamAPI_ISteamUtils_GetAPICallResult(
			self.0,
			call,
			transmute(&mut result),
			size_of::<T>() as u32,
			T::CALLBACK_ID,
			&mut b as *mut bool
		)});

		if b {
			Ok(result)
		} else {
			Err(unsafe { SteamAPI_ISteamUtils_GetAPICallFailureReason(self.0, call) })
		}
	}
}

pub struct Client(*mut ClientImpl, User, Pipe);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientError {
	NoSteam,
	NoSteamPipe,
	NoSteamClient,
}

impl fmt::Display for ClientError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::ClientError::*;
		write!(f, "{}", match *self {
			NoSteam       => "laspad could not initialize SteamAPI",
			NoSteamPipe   => "laspad could not create a pipe to steam.exe",
			NoSteamClient => "laspad could not create SteamClient",
		})
	}
}

impl Client {
	pub fn new() -> Result<Self, ClientError> {
		if ! unsafe {SteamAPI_Init()} {
			return Err(ClientError::NoSteam);
		};

		let pipe = unsafe { SteamAPI_GetHSteamPipe() };
		let user = unsafe { SteamAPI_GetHSteamUser() };

		if pipe == Pipe(0) {
			return Err(ClientError::NoSteamPipe);
		};

		let client: *mut ClientImpl = unsafe { transmute(SteamInternal_CreateInterface(transmute("SteamClient017\0".as_ptr()))) };

		if client as usize == 0 {
			Err(ClientError::NoSteamClient)
		} else {
			Ok(Client(client, user, pipe))
		}
	}

	pub fn remote_storage(&self) -> Result<RemoteStorage, ()> {
		//println!("{:?}, {:?}, {:?}", self.0, self.1, self.2);
		let storage = unsafe { SteamAPI_ISteamClient_GetISteamRemoteStorage(self.0, self.1, self.2, transmute("STEAMREMOTESTORAGE_INTERFACE_VERSION014\0".as_ptr())) };
		if !storage.is_null() {
			Ok(RemoteStorage(storage))
		} else {
			Err(())
		}
	}

	pub fn utils(&self) -> Result<Utils, ()> {
		let utils = unsafe { SteamAPI_ISteamClient_GetISteamUtils(self.0, self.2, transmute("SteamUtils009\0".as_ptr())) };
		if !utils.is_null() {
			Ok(Utils(utils))
		} else {
			Err(())
		}
	}
}
