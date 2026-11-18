@echo off
cd /d "%~dp0"
python "%~dp0src\launcher.py" %*
if errorlevel 1 pause
