@echo off

setlocal ENABLEEXTENSIONS
set KEY_NAME="HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Steam App 4920"
set VALUE_NAME=InstallLocation

FOR /F "usebackq skip=1 tokens=1-2,*" %%A IN (`REG QUERY %KEY_NAME% /v %VALUE_NAME%`) DO (
	set InstallLocation=%%C\x64
)

if defined InstallLocation (
	REM
) else (
	@echo NS2 not found. Please install manually.
	pause
	goto :eof
)

copy "%InstallLocation%\steam_api64.dll"
setx /m PATH "%PATH%;%cd%"
@echo laspad has been successfully installed into the current folder!
@echo Please remove this folder from your PATH environment variable to properly uninstall.

pause
