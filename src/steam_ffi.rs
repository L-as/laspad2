#[repr(u32)]
pub enum Result {
	OK                                      = 1,
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

#[repr(i64)]
pub enum FailureReason {
	None               = -1,
	SteamGone          = 0,
	NetworkFailure     = 1,
	InvalidHandle      = 2,
	MismatchedCallback = 3,
}

#[repr(C)]
pub struct Callback(u64);

#[repr(C)]
pub struct Item(pub u64);

enum RemoteStorageImpl {}
#[repr(C)]
pub struct RemoteStorage(*mut RemoteStorageImpl);

enum UtilsImpl {}
#[repr(C)]
pub struct Utils(*mut UtilsImpl);

#[repr(C)]
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
	elements: *const *const u8,
	length:   u32,
}

#[repr(packed)]
struct PublishFileResult {
	result:           Result,
	item:             Item,
	accept_agreement: bool,
}

impl PublishFileResult {
	const INITIAL_PUBLISH_CALLBACK_ID: u64 = 1309;
	const UPDATE_PUBLISH_CALLBACK_ID:  u64 = 1316;
}

extern {
	fn SteamAPI_Init() -> bool;

	fn SteamRemoteStorage() -> RemoteStorage;
	fn SteamUtils()         -> Utils;

	fn SteamAPI_ISteamRemoteStorage_PublishWorkshopFile(a: RemoteStorage, b: *const u8, c: *const u8, d: u32, e: *const u8, f: *const u8, g: Visibility, h: *const Strings, i: FileType) -> Callback;

	fn SteamAPI_ISteamRemoteStorage_FileWrite(a:  RemoteStorage, b: *const u8, c: *const u8, d: u32) -> bool;
	fn SteamAPI_ISteamRemoteStorage_FileDelete(a: RemoteStorage, b: *const u8) -> bool;

	fn SteamAPI_ISteamRemoteStorage_CreatePublishedFileUpdateRequest(a: RemoteStorage, b: Item)         -> UpdateHandle;
	fn SteamAPI_ISteamRemoteStorage_CommitPublishedFileUpdate(a:        RemoteStorage, b: UpdateHandle) -> Callback;

	fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFileFile(a:                 RemoteStorage, b: UpdateHandle, c: *const u8)      -> bool;
	fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFilePreviewFile(a:          RemoteStorage, b: UpdateHandle, c: *const u8)      -> bool;
	fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFileDescription(a:          RemoteStorage, b: UpdateHandle, c: *const u8)      -> bool;
	fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFileSetChangeDescription(a: RemoteStorage, b: UpdateHandle, c: *const u8)      -> bool;
	fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFileTags(a:                 RemoteStorage, b: UpdateHandle, c: *const Strings) -> bool;
	fn SteamAPI_ISteamRemoteStorage_UpdatePublishedFileTitle(a:                RemoteStorage, b: UpdateHandle, c: *const u8)      -> bool;

	fn SteamAPI_ISteamUtils_IsAPICallCompleted(a:      Utils, b: Callback, c: *mut bool) -> bool;
	fn SteamAPI_ISteamUtils_GetAPICallResult(a:        Utils, b: Callback, c: *mut u8,   d: u32, e: u32, f: *mut bool) -> bool;
	fn SteamAPI_ISteamUtils_GetAPICallFailureReason(a: Utils, b: Callback) -> FailureReason;
}
