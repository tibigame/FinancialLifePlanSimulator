@echo off
setlocal
pushd "%~dp0"
if errorlevel 1 exit /b 1

rem Prefer Bun, including its default per-user installation.
where bun >nul 2>nul
if not errorlevel 1 goto use_bun
if exist "%USERPROFILE%\.bun\bin\bun.exe" (
    set "PATH=%USERPROFILE%\.bun\bin;%PATH%"
    goto use_bun
)
where yarn >nul 2>nul
if not errorlevel 1 goto use_yarn
echo ERROR: Install Bun or Yarn before building. 1>&2
goto failed

:use_bun
call bun install
if errorlevel 1 goto failed
call bun run build
if errorlevel 1 goto failed
goto build_app

:use_yarn
call yarn install
if errorlevel 1 goto failed
call yarn run build
if errorlevel 1 goto failed

:build_app
rem Keep the output location predictable even with a global Cargo target-dir.
set "CARGO_TARGET_DIR=%CD%\src-tauri\target"
rem The frontend was built above with the available package manager.
call node_modules\.bin\tauri.cmd build --debug --no-bundle --config "{\"build\":{\"beforeBuildCommand\":\"\"}}"
if errorlevel 1 goto failed
if not exist "src-tauri\target\debug\financiallifeplansimulator.exe" (
    echo ERROR: The expected executable was not produced. 1>&2
    goto failed
)
copy /y "src-tauri\target\debug\financiallifeplansimulator.exe" "financiallifeplansimulator_debug.exe" >nul
if errorlevel 1 goto failed
echo Built: %CD%\financiallifeplansimulator_debug.exe
popd
exit /b 0

:failed
echo ERROR: Build or copy failed. 1>&2
popd
exit /b 1
