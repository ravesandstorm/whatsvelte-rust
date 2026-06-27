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

:: Check for uncommitted local changes (including untracked files)
set "UNCOMMITTED="
for /f "delims=" %%i in ('git status --porcelain') do set "UNCOMMITTED=true"
if "!UNCOMMITTED!"=="true" (
    echo X Error: Working directory is not clean. Please commit or stash local changes before releasing.
    exit /b 1
)

:: Ensure local and remote are in sync
echo Fetching latest remote state...
git fetch -q
if %errorlevel% neq 0 (
    echo X Error: Failed to fetch from remote!
    exit /b 1
)

for /f %%i in ('git rev-parse HEAD') do set "LOCAL_HEAD=%%i"
for /f %%i in ('git rev-parse @{u} 2^>nul') do set "REMOTE_HEAD=%%i"

if "%REMOTE_HEAD%"=="" (
    echo X Error: No upstream tracking branch configured for the current branch!
    exit /b 1
)

if not "%LOCAL_HEAD%"=="%REMOTE_HEAD%" (
    echo X Error: Local and remote branches are out of sync. Please pull/push before releasing.
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
:: Note: Because we verified the tree is clean, `git add` will now ONLY stage the tauri.conf.json update
git add "%conf%" && ^
git commit -m "chore: bump version to v%new%" && ^
git tag "v%new%" && ^
git push origin main && ^
git push origin "v%new%" && ^
echo Success! Released v%new%

exit /b %errorlevel%
