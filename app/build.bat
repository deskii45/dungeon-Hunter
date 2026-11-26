    @echo off
rem Build the game with cargo.
rem Usage: build.bat                -> build
rem        build.bat run [ARGS...]  -> build, then run the game
setlocal
pushd "%~dp0"
set "CARGO_EXE=%USERPROFILE%\.cargo\bin\cargo.exe"
where cargo >nul 2>nul && set "CARGO_EXE=cargo"
if /I "%~1"=="run" goto :run
call "%CARGO_EXE%" build --offline
if errorlevel 1 goto :fail
echo BUILD OK: %~dp0target\debug\dungeon_hunter.exe
popd
exit /b 0

:run
shift
call "%CARGO_EXE%" build --offline
if errorlevel 1 goto :fail
start "" "%~dp0target\debug\dungeon_hunter.exe" %*
popd
exit /b 0

:fail
echo BUILD FAILED
popd
exit /b 1
