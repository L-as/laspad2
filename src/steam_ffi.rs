#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SteamResult(u32);

impl From<SteamResult> for Result<(), GeneralError> {
	fn from(sr: SteamResult) -> Self {
		use std::mem::transmute;

		match sr {
			SteamResult(1) => Ok(()),
			SteamResult(n) => Err(unsafe{transmute(n)}),
		}
	}
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneralError {
	Fail                                    = 2,
	NoConnection                            = 3,
	InvalidPassword                         = 5,
	LoggedInElsewhere                       = 6,
	InvalidProtocolVer                      = 7,
	InvalidParam                            = 8,
	FileNotFound                            = 9,
	Busy                                    = 10,
	InvalidState                            = 11,
	InvalidName                             = 12,
	InvalidEmail                            = 13,
	DuplicateName                           = 14,
	AccessDenied                            = 15,
	Timeout                                 = 16,
	Banned                                  = 17,
	AccountNotFound                         = 18,
	InvalidSteamID                          = 19,
	ServiceUnavailable                      = 20,
	NotLoggedOn                             = 21,
	Pending                                 = 22,
	EncryptionFailure                       = 23,
	InsufficientPrivilege                   = 24,
	LimitExceeded                           = 25,
	Revoked                                 = 26,
	Expired                                 = 27,
	AlreadyRedeemed                         = 28,
	DuplicateRequest                        = 29,
	AlreadyOwned                            = 30,
	IPNotFound                              = 31,
	PersistFailed                           = 32,
	LockingFailed                           = 33,
	LogonSessionReplaced                    = 34,
	ConnectFailed                           = 35,
	HandshakeFailed                         = 36,
	IOFailure                               = 37,
	RemoteDisconnect                        = 38,
	ShoppingCartNotFound                    = 39,
	Blocked                                 = 40,
	Ignored                                 = 41,
	NoMatch                                 = 42,
	AccountDisabled                         = 43,
	ServiceReadOnly                         = 44,
	AccountNotFeatured                      = 45,
	AdministratorOK                         = 46,
	ContentVersion                          = 47,
	TryAnotherCM                            = 48,
	PasswordRequiredToKickSession           = 49,
	AlreadyLoggedInElsewhere                = 50,
	Suspended                               = 51,
	Cancelled                               = 52,
	DataCorruption                          = 53,
	DiskFull                                = 54,
	RemoteCallFailed                        = 55,
	PasswordUnset                           = 56,
	ExternalAccountUnlinked                 = 57,
	PSNTicketInvalid                        = 58,
	ExternalAccountAlreadyLinked            = 59,
	RemoteFileConflict                      = 60,
	IllegalPassword                         = 61,
	SameAsPreviousValue                     = 62,
	AccountLogonDenied                      = 63,
	CannotUseOldPassword                    = 64,
	InvalidLoginAuthCode                    = 65,
	AccountLogonDeniedNoMail                = 66,
	HardwareNotCapableOfIPT                 = 67,
	IPTInitError                            = 68,
	ParentalControlRestricted               = 69,
	FacebookQueryError                      = 70,
	ExpiredLoginAuthCode                    = 71,
	IPLoginRestrictionFailed                = 72,
	AccountLockedDown                       = 73,
	AccountLogonDeniedVerifiedEmailRequired = 74,
	NoMatchingURL                           = 75,
	BadResponse                             = 76,
	RequirePasswordReEntry                  = 77,
	ValueOutOfRange                         = 78,
	UnexpectedError                         = 79,
	Disabled                                = 80,
	InvalidCEGSubmission                    = 81,
	RestrictedDevice                        = 82,
	RegionLocked                            = 83,
	RateLimitExceeded                       = 84,
	AccountLoginDeniedNeedTwoFactor         = 85,
	ItemDeleted                             = 86,
	AccountLoginDeniedThrottle              = 87,
	TwoFactorCodeMismatch                   = 88,
	TwoFactorActivationCodeMismatch         = 89,
	AccountAssociatedToMultiplePartners     = 90,
	NotModified                             = 91,
	NoMobileDevice                          = 92,
	TimeNotSynced                           = 93,
	SmsCodeFailed                           = 94,
	AccountLimitExceeded                    = 95,
	AccountActivityLimitExceeded            = 96,
	PhoneActivityLimitExceeded              = 97,
	RefundToWallet                          = 98,
	EmailSendFailure                        = 99,
	NotSettled                              = 100,
	NeedCaptcha                             = 101,
	GSLTDenied                              = 102,
	GSOwnerDenied                           = 103,
	InvalidItemType                         = 104,
	IPBanned                                = 105,
	GSLTExpired                             = 106,
	InsufficientFunds                       = 107,
	TooManyPending                          = 108,
	NoSiteLicensesFound                     = 109,
	WGNetworkSendExceeded                   = 110,
}

impl fmt::Display for GeneralError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self, f)
	}
}

impl error::Error for GeneralError {
	fn description(&self) -> &str {
		"<N/A>"
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UpdateHandle(u64);

#[repr(u32)]
enum Visibility {
	Public,
	FriendsOnly,
	Private
}

#[repr(u32)]
enum FileType {
	Community,
	Microtransaction,
	Collection,
}

#[repr(C)]
#[repr(packed)]
struct Strings {
	elements: *const *const i8,
	length:   i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct User(i32);

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pipe(i32);

#[cfg_attr(unix,    link(name = "steam_api"))]
#[cfg_attr(windows, link(name = "steam_api64"))]
#[no_mangle]
extern "C" {
	type RemoteStorageImpl;
	type UtilsImpl;
	type ClientImpl;

	fn SteamAPI_Init() -> bool;
	fn SteamAPI_Shutdown();

	fn SteamAPI_ISteamRemoteStorage_PublishWorkshopFile(a: *mut RemoteStorageImpl, b: *const i8, c: *const i8, d: u32, e: *const i8, f: *const i8, g: Visibility, h: *const Strings, i: FileType) -> APICall;

	fn SteamAPI_ISteamRemoteStorage_FileWrite(a:  *mut RemoteStorageImpl, b: *const i8, c: *const u8, d: u32) -> bool;
	fn SteamAPI_ISteamRemoteStorage_FileDelete(a: *mut RemoteStorageImpl, b: *const i8) -> bool;

	fn SteamAPI_ISteamRemoteStorage_CreatePublishedFileUpdateRequest(a: *mut RemoteStorageImpl, b: Item)         -> UpdateHandle;
	fn SteamAPI_ISteamRemoteStorage_CommitPublishedFileUpdate(a:        *mut RemoteStorageImpl, b: UpdateHandle) -> APICall;

	fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFileFile(a:                 *mut RemoteStorageImpl, b: UpdateHandle, c: *const i8)      -> bool;
	fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFilePreviewFile(a:          *mut RemoteStorageImpl, b: UpdateHandle, c: *const i8)      -> bool;
	fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFileDescription(a:          *mut RemoteStorageImpl, b: UpdateHandle, c: *const i8)      -> bool;
	fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFileSetChangeDescription(a: *mut RemoteStorageImpl, b: UpdateHandle, c: *const i8)      -> bool;
	fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFileTags(a:                 *mut RemoteStorageImpl, b: UpdateHandle, c: *const Strings) -> bool;
	fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFileTitle(a:                *mut RemoteStorageImpl, b: UpdateHandle, c: *const i8)      -> bool;

	fn SteamAPI_ISteamUtils_IsAPICallCompleted(a:      *mut UtilsImpl, b: APICall, c: *mut bool) -> bool;
	fn SteamAPI_ISteamUtils_GetAPICallResult(a:        *mut UtilsImpl, b: APICall, c: *mut u8,   d: u32, e: u32, f: *mut bool) -> bool;
	fn SteamAPI_ISteamUtils_GetAPICallFailureReason(a: *mut UtilsImpl, b: APICall) -> APICallFailureReason;

	fn SteamAPI_ISteamClient_GetISteamRemoteStorage(a: *const ClientImpl, b: User, c: Pipe, d: *const i8) -> *mut RemoteStorageImpl;
	fn SteamAPI_ISteamClient_GetISteamUtils(a:         *const ClientImpl, b: Pipe, c: *const i8)          -> *mut UtilsImpl;

	fn SteamAPI_GetHSteamUser() -> User;
	fn SteamAPI_GetHSteamPipe() -> Pipe;

	fn SteamInternal_CreateInterface(a: *const i8) -> *mut u8;
}
