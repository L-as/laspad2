#!/bin/sh

cd "$(dirname "$0")"

set -ex

if test "x$LASPAD_DEBUG" = xtrue
then
	cargo build
	exe=target/debug/laspad
else
	cargo build --release
	exe=target/release/laspad
fi

rm -rf target/appdir
mkdir  target/appdir

cp $exe                     target/appdir/AppRun
cp laspad.desktop           target/appdir/laspad.desktop
cp icon.png                 target/appdir/laspad.png
cp 3rdparty/libsteam_api.so target/appdir/libsteam_api.so
cp steam_appid.txt          target/appdir/steam_appid.txt

appimagetool target/appdir laspad.AppImage
