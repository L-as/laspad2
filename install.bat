@echo off

setlocal ENABLEEXTENSIONS

cargo build --release

copy "3rdparty\steam_api64.dll"  "%HOMEPATH%\.cargo\bin\"
copy "steam_appid.txt"           "%HOMEPATH%\.cargo\bin\"
copy "target\release\laspad.exe" "%HOMEPATH%\.cargo\bin\"

rem Stolen from https://superuser.com/a/455383
set SCRIPT="%TEMP%\%RANDOM%-%RANDOM%-%RANDOM%-%RANDOM%.vbs"

echo Set oWS = WScript.CreateObject("WScript.Shell") >> %SCRIPT%
echo sLinkFile = "%USERPROFILE%\Desktop\laspad.lnk" >> %SCRIPT%
echo Set oLink = oWS.CreateShortcut(sLinkFile) >> %SCRIPT%
echo oLink.TargetPath = "%HOMEPATH%\.cargo\bin\laspad.exe" >> %SCRIPT%
echo oLink.Save >> %SCRIPT%

cscript /nologo %SCRIPT%
del %SCRIPT%
@echo laspad has been successfully installed into ~/.cargo/bin (also in your PATH)

pause
