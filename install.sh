#!/usr/bin/env bash

echo -E '
Set INSTALL_DIR to where laspad should be installed. Default is /usr/local.
'

cd "$(dirname "$0")"

set -ex

INSTALL_DIR="$(realpath -s "${INSTALL_DIR:=/usr/local}")"
mkdir -p "$INSTALL_DIR/lib/laspad"

test "x$LASPAD_DEBUG" = xtrue && {
	cargo build &&
	cp target/debug/laspad steam_appid.txt 3rdparty/libsteam_api.so "$INSTALL_DIR/lib/laspad/" &&
	ln -sf "$INSTALL_DIR/lib/laspad/laspad" "$INSTALL_DIR/bin/laspad"
	exit $?
} || {
	cargo build --release &&
	cp target/release/laspad steam_appid.txt 3rdparty/libsteam_api.so "$INSTALL_DIR/lib/laspad/" &&
	ln -sf "$INSTALL_DIR/lib/laspad/laspad" "$INSTALL_DIR/bin/laspad"
	exit $?
}

