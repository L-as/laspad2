@echo off

setlocal ENABLEEXTENSIONS

cargo build --release

del laspad.7z
md laspad

copy "3rdparty\steam_api64.dll"  laspad
copy "assets\steam_appid.txt"    laspad
copy "target\release\laspad.exe" laspad

7z a laspad.7z laspad
