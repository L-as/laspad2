with import <nixpkgs> {};
let libsteam_api = fetchurl {
	url = https://raw.githubusercontent.com/rlabrecque/Steamworks.NET/master/Plugins/x86_64/libsteam_api.so;
	sha256 = "04rqg7d4vzi9kf948bm8dw1q2z6gn3qqxma5ajahbcd9dhms0bs0";
};
in stdenv.mkDerivation rec {
	name = "laspad-${version}";
	version = "2.0.0";

	src = ./.;

	inherit libsteam_api;
	buildInputs = [latest.rustChannels.nightly.rust openssl gcc pkgconfig];

	buildPhase = ''
		env RUST_BACKTRACE=1 cargo rustc -- -C link-arg=-Wl,"$libsteam_api"
	'';

	installPhase = ''
		mkdir -p "$out/bin"
		mv target/debug/laspad "$out/bin"
		cp assets/steam_appid.txt "$out/bin"
	'';

	postFixup = ''
		patchelf --remove-needed libsteam_api.so "$out/bin/laspad"
		patchelf --add-needed "$libsteam_api" "$out/bin/laspad"
	'';

	dontFixLibtool = true;
}
