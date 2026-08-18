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

:: Locate Cargo.toml
for /f "tokens=*" %%i in ('git rev-parse --show-toplevel 2^>nul') do set "GIT_ROOT=%%i"
if "%GIT_ROOT%"=="" (
    echo X Error: Not a git repository!
    exit /b 1
)

set "conf=%GIT_ROOT%\Cargo.toml"

if not exist "%conf%" (
    echo X Error: %conf% not found!
    exit /b 1
)

echo Building and releasing for %part%...
exit /b 0
