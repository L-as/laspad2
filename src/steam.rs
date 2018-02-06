#![allow(dead_code)]

use std::result::Result;

include!("steam_ffi.rs");

pub fn init() -> Result<(), ()> {
	let ok = unsafe { SteamAPI_Init() };
	if ok {
		Ok(())
	} else {
		Err(())
	}
}

impl RemoteStorage {
	fn new() -> Self {
		let this = unsafe { SteamRemoteStorage() };
		assert_ne!(this.0 as usize, 0, "Please call steam::init first!");
		this
	}
}

impl Utils {
	fn new() -> Self {
		let this = unsafe { SteamUtils() };
		assert_ne!(this.0 as usize, 0, "Please call steam::init first!");
		this
	}
}
