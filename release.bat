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

:: Locate the git repository root
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

set "conf=%GIT_ROOT%\slint-client\Cargo.toml"

:: Check if the file actually exists
if not exist "%conf%" (
    echo X Error: %conf% not found!
    exit /b 1
)

:: Current version: the first `version = "x.y.z"` line is the [package] version
set "cur="
for /f "tokens=2 delims==" %%i in ('findstr /b /c:"version = " "%conf%"') do (
    if not defined cur set "cur=%%~i"
)
set "cur=%cur: =%"
set "cur=%cur:"=%"

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

:: Rewrite only the first (package) version line
powershell -NoProfile -Command "$p='%conf%'; $done=$false; $out=Get-Content $p | ForEach-Object { if (-not $done -and $_ -match '^version = \"') { $done=$true; 'version = \"%new%\"' } else { $_ } }; Set-Content -Path $p -Value $out"
if %errorlevel% neq 0 (
    echo X Error: Failed to update %conf%.
    exit /b 1
)

:: Refresh Cargo.lock's entry for the app itself (no dependency changes)
pushd "%GIT_ROOT%\slint-client"
cargo update --workspace --offline -q
set "CARGO_ERR=%errorlevel%"
popd
if not "%CARGO_ERR%"=="0" (
    echo X Error: cargo update failed.
    exit /b 1
)

:: Git Operations
:: Because we verified the tree is clean, `git add` will ONLY stage the version updates
git add "%conf%" "%GIT_ROOT%\slint-client\Cargo.lock" && ^
git commit -m "chore: bump version to v%new%" && ^
git tag "v%new%" && ^
git push origin main && ^
git push origin "v%new%" && ^
echo Success! Released v%new%

exit /b %errorlevel%
