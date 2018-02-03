#/usr/bin/env bash

LD_RUN_PATH='$ORIGIN/ns2:$ORIGIN' dub build -b release --compiler=ldc --force
tar -cf laspad.tar laspad steam_appid.txt install.sh
gzip -f9 laspad.tar
