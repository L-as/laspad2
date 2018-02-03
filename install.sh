#/usr/bin/env bash

echo -E '
Set INSTALL_DIR to where laspad should be installed. Default is /usr/local.
'

set -e
set -x

INSTALL_DIR="$(realpath -s "${INSTALL_DIR:=/usr/local}")"
mkdir -p "$INSTALL_DIR/lib/laspad"

declare NS2_DIR

sed -En 's:BaseInstallFolder_.*?"\s*"(.*?)":\1:p' < ~/.local/share/Steam/config/config.vdf |
	while read dir
	do
		# I have no idea why the " is included
		dir="${dir:1}/steamapps/common/Natural Selection 2"
		test -e "$dir" && {
			ln -sf "$dir/x64" "$INSTALL_DIR/lib/laspad/ns2"
			NS2_DIR="$dir"
			exit 0
		}
	done

test -e "$NS2_DIR" || test -e "${NS2_DIR=$HOME/.local/share/Steam/steamapps/common/Natural Selection 2}" || {
	echo >&2 "Could not find NS2 installation directory! Please submit an issue here: https://github.com/Laaas/laspad/issues"
	exit 1
}

ln -sf "$NS2_DIR/x64/libsteam_api.so" "libsteam_api.so"
test -e laspad || which dub && LD_RUN_PATH='$ORIGIN/ns2:$ORIGIN' dub build -b release --compiler=ldc --force || {
	echo >&2 "Can not build executable and no executable presupplied!"
	exit 1
}
cp laspad          "$INSTALL_DIR/lib/laspad/"
cp steam_appid.txt "$INSTALL_DIR/lib/laspad/"
ln -sf "$INSTALL_DIR/lib/laspad/laspad" "$INSTALL_DIR/bin/laspad"
ln -sf "$HOME/.local/share/Steam/steamapps/common/Natural Selection 2/x64" "$INSTALL_DIR/lib/laspad/ns2"

