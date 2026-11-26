    @echo off
rem Build the game with cargo.
rem Usage: build.bat                -> build debug
rem        build.bat run [ARGS...]  -> build debug, then run the game
rem        build.bat release        -> build portable release exe (static CRT) and copy to repo root
setlocal
pushd "%~dp0"
set "CARGO_EXE=%USERPROFILE%\.cargo\bin\cargo.exe"
where cargo >nul 2>nul && set "CARGO_EXE=cargo"
if /I "%~1"=="run" goto :run
if /I "%~1"=="release" goto :release
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

:release
set "RUSTFLAGS=-C target-feature=+crt-static"
call "%CARGO_EXE%" build --release --offline
if errorlevel 1 goto :fail
copy /Y "%~dp0target\release\dungeon_hunter.exe" "%~dp0..\dungeon_hunter.exe" >nul
if errorlevel 1 goto :fail
echo RELEASE BUILD OK: %~dp0..\dungeon_hunter.exe
popd
exit /b 0

:fail
echo BUILD FAILED
popd
exit /b 1
