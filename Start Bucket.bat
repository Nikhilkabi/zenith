@echo off
cd /d "%~dp0"
start "Bucket Dev" /MIN cmd /c "npm run tauri:dev"
echo Starting Bucket in development mode...
echo Close the minimized console window to stop the dev server.
