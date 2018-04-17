@echo off

setlocal ENABLEEXTENSIONS

cargo build --release

del laspad.7z
md laspad

copy "3rdparty\steam_api64.dll"  laspad
copy "steam_appid.txt"           laspad
copy "laspad.bat"                laspad
copy "target\release\laspad.exe" laspad\laspad-gui.exe

7z a laspad.7z laspad