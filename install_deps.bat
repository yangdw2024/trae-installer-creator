@echo off
cd /d "%~dp0"
echo Installing Trae Installer Creator dependencies...
python -m pip install -r "%~dp0requirements.txt"
if errorlevel 1 (
    echo Failed to install dependencies. Please ensure Python 3.8+ and pip are installed.
    pause
    exit /b 1
)
echo Dependencies installed successfully.
pause
