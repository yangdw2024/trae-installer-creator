#!/usr/bin/env python3
import sys
import os
import subprocess


def check_python():
    try:
        import tkinter
    except ImportError:
        return False, "tkinter module not found. Please install Python with tkinter support."
    
    try:
        import win32com.client
    except ImportError:
        return False, "pywin32 module not found. Run: pip install pywin32"
    
    return True, None


def find_python():
    python_exe = None
    
    if sys.executable and os.path.exists(sys.executable):
        python_exe = sys.executable
    
    if not python_exe:
        for path in os.environ.get('PATH', '').split(';'):
            candidate = os.path.join(path, 'python.exe')
            if os.path.exists(candidate):
                python_exe = candidate
                break
    
    if not python_exe:
        for drive in ['D:', 'C:']:
            candidate = os.path.join(drive, 'Python311', 'python.exe')
            if os.path.exists(candidate):
                python_exe = candidate
                break
            candidate = os.path.join(drive, 'Python310', 'python.exe')
            if os.path.exists(candidate):
                python_exe = candidate
                break
    
    return python_exe


def main():
    script_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    gui_script = os.path.join(script_dir, 'src', 'gui', 'main_window.py')
    
    if not os.path.exists(gui_script):
        print(f"Error: GUI script not found at {gui_script}")
        input("Press Enter to exit...")
        return
    
    ok, error = check_python()
    if not ok:
        print(f"Error: {error}")
        print("Trying to install pywin32...")
        try:
            subprocess.run([sys.executable, '-m', 'pip', 'install', 'pywin32'], 
                         check=True, capture_output=True)
            print("pywin32 installed successfully.")
        except Exception as e:
            print(f"Failed to install pywin32: {e}")
            print("Please run 'pip install pywin32' manually.")
            input("Press Enter to exit...")
            return
    
    print(f"Starting GUI: {gui_script}")
    subprocess.run([sys.executable, gui_script])


if __name__ == '__main__':
    main()