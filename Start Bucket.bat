@echo off
set "EXE=%LOCALAPPDATA%\Programs\Bucket\Zenith.exe"
if not exist "%EXE%" set "EXE=%LOCALAPPDATA%\Programs\Bucket\Bucket.exe"
if exist "%EXE%" (
  start "" "%EXE%"
  exit /b 0
)
echo Zenith is not installed as an app yet. Building needs a one-time compile.
echo Run: npm run install:local
pause
