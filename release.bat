@echo off
setlocal enabledelayedexpansion

:: Exit immediately if any command fails
set "ERRORLEVEL=0"

:: Accept terminal argument and check if empty
if "%~1"=="" (
    echo usage: release.bat ^<major^|minor^|patch^>
    exit /b 1
)

:: Run the core logic
call :bump_version "%~1"
exit /b %errorlevel%

:bump_version
set "part=%~1"

:: Locate the tauri.conf.json file relative to the git repository root
for /f "tokens=*" %%i in ('git rev-parse --show-toplevel 2^>nul') do set "GIT_ROOT=%%i"
if "%GIT_ROOT%"=="" (
    echo X Error: Not a git repository!
    exit /b 1
)

set "conf=%GIT_ROOT%\src-tauri\tauri.conf.json"

:: Check if the file actually exists
if not exist "%conf%" (
    echo X Error: %conf% not found!
    exit /b 1
)

:: Get current version using jq
for /f "tokens=*" %%i in ('jq -r ".version" "%conf%"') do set "cur=%%i"

:: Parse major, minor, and patch values
for /f "tokens=1,2,3 delims=." %%a in ("%cur%") do (
    set /a major=%%a
    set /a minor=%%b
    set /a patch=%%c
)

:: Increment the version based on input choice
if "%part%"=="major" (
    set /a major+=1
    set minor=0
    set patch=0
) else if "%part%"=="minor" (
    set /a minor+=1
    set patch=0
) else if "%part%"=="patch" (
    set /a patch+=1
) else (
    echo usage: release.bat ^<major^|minor^|patch^>
    exit /b 1
)

set "new=%major%.%minor%.%patch%"

:: Safely update the JSON file using jq arguments
jq --arg v "%new%" ".version = $v" "%conf%" > "%conf%.tmp"
if %errorlevel% neq 0 (
    echo X Error: Failed to update JSON using jq.
    if exist "%conf%.tmp" del "%conf%.tmp"
    exit /b 1
)

move /y "%conf%.tmp" "%conf%" >nul

:: Git Operations
git add "%conf%" && ^
git commit -m "chore: bump version to v%new%" && ^
git tag "v%new%" && ^
git push origin main && ^
git push origin "v%new%" && ^
echo Success! Released v%new%

exit /b %errorlevel%
