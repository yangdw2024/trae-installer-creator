import os
import sys
import shutil
import json
import struct
import winreg
from typing import Dict, List, Optional


class UninstallBuilder:
    def __init__(self):
        self.uninstall_config: Dict[str, str] = {}
        self.installed_files: List[str] = []
        self.install_dir: str = ''
    
    def generate(self, install_dir: str, installed_files: List[str], config: Dict[str, str]) -> str:
        self.install_dir = install_dir
        self.installed_files = installed_files
        self.uninstall_config = config
        
        app_name = config.get('AppName', 'My Program')
        
        uninstall_dir = os.path.join(install_dir, 'uninstall')
        os.makedirs(uninstall_dir, exist_ok=True)
        
        uninstall_exe = os.path.join(uninstall_dir, 'uninstall.exe')
        self._create_uninstall_script(uninstall_dir)
        
        self._register_uninstaller(app_name, uninstall_exe)
        
        return uninstall_exe
    
    def _create_uninstall_script(self, uninstall_dir: str) -> None:
        uninstall_data = {
            'install_dir': self.install_dir,
            'installed_files': self.installed_files,
            'config': self.uninstall_config
        }
        
        data_path = os.path.join(uninstall_dir, 'uninstall_data.json')
        with open(data_path, 'w', encoding='utf-8') as f:
            json.dump(uninstall_data, f, ensure_ascii=False, indent=2)
        
        script_code = '''import sys
import os
import json
import shutil
import winreg

def uninstall():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    data_path = os.path.join(script_dir, 'uninstall_data.json')
    
    with open(data_path, 'r', encoding='utf-8') as f:
        data = json.load(f)
    
    install_dir = data.get('install_dir', '')
    installed_files = data.get('installed_files', [])
    config = data.get('config', {})
    
    app_name = config.get('AppName', 'My Program')
    
    confirm = input(f'Are you sure you want to uninstall {app_name}? (y/n): ')
    if confirm.lower() != 'y':
        print('Uninstallation cancelled')
        return
    
    for filepath in installed_files:
        full_path = os.path.join(install_dir, filepath)
        if os.path.exists(full_path):
            try:
                os.remove(full_path)
                print(f'Removed: {full_path}')
            except Exception as e:
                print(f'Failed to remove {full_path}: {e}')
    
    for root, dirs, files in os.walk(install_dir, topdown=False):
        for dir in dirs:
            dir_path = os.path.join(root, dir)
            try:
                os.rmdir(dir_path)
                print(f'Removed directory: {dir_path}')
            except:
                pass
    
    try:
        os.rmdir(install_dir)
        print(f'Removed installation directory: {install_dir}')
    except:
        print(f'Could not remove installation directory: {install_dir}')
    
    _unregister_uninstaller(app_name)
    
    print(f'{app_name} has been uninstalled successfully')

def _unregister_uninstaller(app_name):
    try:
        key_path = f'Software\\\\Microsoft\\\\Windows\\\\CurrentVersion\\\\Uninstall\\\\{app_name}'
        winreg.DeleteKey(winreg.HKEY_CURRENT_USER, key_path)
    except Exception:
        pass
    
    try:
        key_path = f'Software\\\\Microsoft\\\\Windows\\\\CurrentVersion\\\\Uninstall\\\\{app_name}'
        winreg.DeleteKey(winreg.HKEY_LOCAL_MACHINE, key_path)
    except Exception:
        pass

if __name__ == '__main__':
    uninstall()
'''
        
        script_path = os.path.join(uninstall_dir, 'uninstall.py')
        with open(script_path, 'w', encoding='utf-8') as f:
            f.write(script_code)
    
    def _register_uninstaller(self, app_name: str, uninstall_exe: str) -> None:
        try:
            key_path = f'Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{app_name}'
            key = winreg.CreateKey(winreg.HKEY_CURRENT_USER, key_path)
            
            winreg.SetValueEx(key, 'DisplayName', 0, winreg.REG_SZ, app_name)
            winreg.SetValueEx(key, 'UninstallString', 0, winreg.REG_SZ, f'"{uninstall_exe}"')
            winreg.SetValueEx(key, 'DisplayVersion', 0, winreg.REG_SZ, self.uninstall_config.get('AppVersion', '1.0'))
            winreg.SetValueEx(key, 'InstallLocation', 0, winreg.REG_SZ, self.install_dir)
            
            winreg.CloseKey(key)
        except Exception as e:
            print(f'Failed to register uninstaller: {e}')