#![allow(dead_code)]
include!("steam_ffi.rs");


impl RemoteStorage {
	fn new() -> Self {
		unsafe { SteamRemoteStorage() }
	}
}

impl Utils {
	fn new() -> Self {
		unsafe { SteamUtils() }
	}
}
