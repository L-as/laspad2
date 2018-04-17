@echo off

setlocal ENABLEEXTENSIONS

cargo build --release

md laspad

copy "3rdparty\steam_api64.dll"  laspad
copy "steam_appid.txt"           laspad
copy "laspad.bat"                laspad
copy "target\release\laspad.exe" laspad
