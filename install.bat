@echo off

setlocal ENABLEEXTENSIONS

cargo build --release

copy "3rdparty\steam_api.dll"    "%HOMEPATH%\.cargo\bin\"
copy "steam_appid.txt"           "%HOMEPATH%\.cargo\bin\"
copy "target\release\laspad.exe" "%HOMEPATH%\.cargo\bin\"
@echo laspad has been successfully installed into ~/.cargo/bin (also in your PATH)

pause
