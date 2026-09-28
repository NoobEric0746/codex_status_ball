@echo off
setlocal

cd /d "%~dp0"

echo [1/2] Building release executable...
cargo build --release --manifest-path "src-tauri\Cargo.toml"
if errorlevel 1 goto build_failed

if not exist "artifacts" mkdir "artifacts"
copy /Y "src-tauri\target\release\app.exe" "artifacts\codex-status-ball.exe" >nul
if errorlevel 1 goto copy_failed

echo.
echo Build completed successfully.
echo Output: %CD%\artifacts\codex-status-ball.exe

exit /b 0

:build_failed
echo.
echo Build failed. Check the error output above.
pause
exit /b 1

:copy_failed
echo.
echo Build succeeded, but copying the executable failed.
echo Source: %CD%\src-tauri\target\release\app.exe
pause
exit /b 1
