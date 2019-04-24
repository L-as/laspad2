{sha256, pkgs ? import <nixpkgs> {}}: with pkgs;
let
	steamworks_net = "https://raw.githubusercontent.com/rlabrecque/Steamworks.NET/2d8e525a51cef096bf97cf101e13354d0c7dfd66/Plugins/x86_64/";
	libsteam_api =
		if stdenv.hostPlatform.config == "x86_64-unknown-linux-gnu"
		then fetchurl {
			url = steamworks_net + "libsteam_api.so";
			sha256 = "0qsn4a3xbwk4n4xhw797ffpi25mzgs07w43y1jb9hw0rxjsm30yz";
		}
		else if stdenv.hostPlatform.config == "x86_64-pc-mingw32"
		then fetchurl {
			url = steamworks_net + "steam_api64.dll";
			sha256 = "0000000000000000000000000000000000000000000000000000";
		}
		else throw "laspad does not support this platform!";
in rustPlatform.buildRustPackage rec {
	name = "laspad-${version}";
	version = "2.0.0";

	src = ./.;

	inherit libsteam_api;
	nativeBuildInputs = [stdenv.cc pkgconfig];
	buildInputs = [openssl_1_1];

	buildPhase = ''
		env RUST_BACKTRACE=1 cargo rustc --release -- -C link-arg=-Wl,"$libsteam_api"
	'';

	installPhase = ''
		mkdir -p "$out/bin"
		mv target/release/laspad "$out/bin"
		cp assets/steam_appid.txt "$out/bin"
	'';

	postFixup = ''
		patchelf --remove-needed libsteam_api.so "$out/bin/laspad"
		patchelf --add-needed "$libsteam_api" "$out/bin/laspad"
	'';

	cargoSha256 = sha256;
}
