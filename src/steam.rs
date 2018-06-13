#![allow(dead_code)]

use std::ffi::CString;
use std::mem::{forget, size_of, transmute, zeroed};
use std::fmt;
use std::ptr;
use std::path::Path;

use failure::*;

include!("steam_ffi.rs");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Item(pub u64);

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct AppID(pub u32);


#[repr(transparent)]
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
pub struct CreateItemResult {
	pub result: SteamResult,
	pub item:   Item,
	/// This is true if the user needs to agree to the legal agreement.
	/// This is doable on the steam workshop's web interface.
	pub legal_agreement_required: bool,
}

impl APICallResult for CreateItemResult {
	const CALLBACK_ID: u32 = 3403;
}

#[repr(packed)]
pub struct DownloadItemResult {
	pub app_id: AppID,
	pub item:   Item,
	pub result: SteamResult,
}

impl APICallResult for DownloadItemResult {
	const CALLBACK_ID: u32 = 3406;
}

#[repr(packed)]
pub struct SubmitItemUpdateResult {
	pub result: SteamResult,
	/// Look in CreateItemResult
	pub legal_agreement_required: bool,
	pub item: Item,
}

impl APICallResult for SubmitItemUpdateResult {
	const CALLBACK_ID: u32 = 3404;
}

#[derive(Clone)]
pub struct UGC(*mut UGCImpl);

pub struct ItemUpdater<'a> {
	handle: UpdateHandle,
	ugc: &'a UGC
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
			strings: tags.as_slice().as_ptr() as *const *const i8,
			count: tags.len() as i32,
		};

		if unsafe { SteamAPI_ISteamUGC_SetItemTags(self.ugc.0, self.handle, &tags as *const Strings) } {
			Ok(&self)
		} else {
			Err(())
		}
	}
	pub fn content(&self, content_path: &Path) -> Result<&Self, ()> {
		if unsafe { SteamAPI_ISteamUGC_SetItemContent(self.ugc.0, self.handle, CString::new(content_path.to_str().unwrap()).unwrap().as_ptr()) } {
			Ok(&self)
		} else {
			Err(())
		}
	}
	pub fn preview(&self, preview_path: &Path) -> Result<&Self, ()> {
		if unsafe { SteamAPI_ISteamUGC_SetItemPreview(self.ugc.0, self.handle, CString::new(preview_path.to_str().unwrap()).unwrap().as_ptr()) } {
			Ok(&self)
		} else {
			Err(())
		}
	}
	pub fn description(&self, description: &str) -> Result<&Self, ()> {
		if unsafe { SteamAPI_ISteamUGC_SetItemDescription(self.ugc.0, self.handle, CString::new(description).unwrap().as_ptr()) } {
			Ok(&self)
		} else {
			Err(())
		}
	}
	pub fn title(&self, title: &str) -> Result<&Self, ()> {
		if unsafe { SteamAPI_ISteamUGC_SetItemTitle(self.ugc.0, self.handle, CString::new(title).unwrap().as_ptr()) } {
			Ok(&self)
		} else {
			Err(())
		}
	}
	pub fn submit(&self, update_note: Option<&str>) -> APICall {
		let ptr = update_note.map(|s| {let s = CString::new(s).unwrap(); let ptr = s.as_ptr(); forget(s); ptr}).unwrap_or(ptr::null());
		unsafe { SteamAPI_ISteamUGC_SubmitItemUpdate(self.ugc.0, self.handle, ptr) }
	}
}

impl UGC {
	pub fn create_item(&mut self) -> APICall {
		unsafe {SteamAPI_ISteamUGC_CreateItem(
			self.0,
			DepotID(4920),
			FileType::Community
		)}
	}

	pub fn update_item<'a>(&'a mut self, app_id: AppID, item: Item) -> ItemUpdater<'a> {
		ItemUpdater {
			handle: unsafe { SteamAPI_ISteamUGC_StartItemUpdate(self.0, app_id, item) },
			ugc: self,
		}
	}
}

#[derive(Clone)]
pub struct Utils(*mut UtilsImpl);

impl Utils {
	pub fn is_apicall_completed(&self, call: APICall) -> bool {
		let mut b = false;
		unsafe { SteamAPI_ISteamUtils_IsAPICallCompleted(self.0, call, &mut b as *mut bool) }
	}

	pub fn get_apicall_result<T: APICallResult>(&self, call: APICall) -> T {
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

#[derive(Clone)]
pub struct Client(*mut ClientImpl);

static STEAM_CLIENT_VERSION: &'static [u8] = b"SteamClient017\0";
static STEAM_UGC_VERSION:    &'static [u8] = b"STEAMUGC_INTERFACE_VERSION010\0";
static STEAM_UTILS_VERSION:  &'static [u8] = b"SteamUtils009\0";

impl Client {
	pub fn new() -> Result<Self, Error> {
		ensure!(unsafe {SteamAPI_Init()}, "Could not initialize SteamAPI");

		let client: *mut ClientImpl = unsafe { transmute(SteamInternal_CreateInterface(STEAM_CLIENT_VERSION.as_ptr() as _)) };

		Ok(Client(client))
	}

	pub fn ugc(&self) -> Result<UGC, Error> {
		let user = unsafe {SteamAPI_GetHSteamUser()};
		let pipe = unsafe {SteamAPI_GetHSteamPipe()};
		let ugc  = unsafe {SteamAPI_ISteamClient_GetISteamUGC(self.0, user, pipe, STEAM_UGC_VERSION.as_ptr() as _)};

		ensure!(!ugc.is_null(), "Could not retrive steam's user generated content API");

		Ok(UGC(ugc))
	}

	pub fn utils(&self) -> Result<Utils, Error> {
		let pipe  = unsafe {SteamAPI_GetHSteamPipe()};
		let utils = unsafe {SteamAPI_ISteamClient_GetISteamUtils(self.0, pipe, STEAM_UTILS_VERSION.as_ptr() as _)};

		ensure!(!utils.is_null(), "Could not retrive steam's utils API");

		Ok(Utils(utils))
	}
}

impl Drop for Client {
	fn drop(&mut self) {
		unsafe {SteamAPI_Shutdown()}
	}
}
