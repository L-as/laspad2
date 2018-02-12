#!/usr/bin/env bash

echo -E '
Set INSTALL_DIR to where laspad should be installed. Default is /usr/local.
'

set -e
set -x

INSTALL_DIR="$(realpath -s "${INSTALL_DIR:=/usr/local}")"
mkdir -p "$INSTALL_DIR/lib/laspad"

cargo build --release
cp target/release/laspad steam_appid.txt libsteam_api.so "$INSTALL_DIR/lib/laspad/"
ln -sf "$INSTALL_DIR/lib/laspad/laspad" "$INSTALL_DIR/bin/laspad"
