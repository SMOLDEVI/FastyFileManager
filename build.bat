@echo off
echo 🦀 Building FastyFileManager for Windows...

cargo build --release

if %ERRORLEVEL% NEQ 0 (
    echo ❌ Build failed!
    exit /b 1
)

echo ✅ Build complete!

REM Copy binary to a local bin directory
if not exist "%USERPROFILE%\bin" mkdir "%USERPROFILE%\bin"
copy /Y "target\release\ffm.exe" "%USERPROFILE%\bin\ffm.exe" >nul

REM Add %USERPROFILE%\bin to user PATH if not already there
echo %PATH% | findstr /C:"%USERPROFILE%\bin" >nul
if %ERRORLEVEL% NEQ 0 (
    setx PATH "%USERPROFILE%\bin;%PATH%"
    echo 📌 Added %USERPROFILE%\bin to PATH (restart terminal to apply)
) else (
    echo 📌 Already in PATH
)

echo 📦 Binary: %USERPROFILE%\bin\ffm.exe
echo 🚀 Run: ffm
