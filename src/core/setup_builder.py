import os
import sys
import shutil
import struct
import tempfile
import json
import zipfile
from typing import Dict, List, Optional
from .script_parser import ScriptParser
from .compressor import Compressor


class SetupBuilder:
    def __init__(self):
        self.parser = ScriptParser()
        self.compressor = Compressor()
        self.setup_config: Dict[str, str] = {}
        self.files: List[Dict[str, str]] = []
        self.icons: List[Dict[str, str]] = []
        self.components: List[Dict[str, str]] = []
        self.tasks: List[Dict[str, str]] = []
        self.code: str = ''
        self.script_dir: str = ''
    
    def load_script(self, script_path: str) -> None:
        self.script_dir = os.path.dirname(os.path.abspath(script_path))
        self.parser.parse_file(script_path)
        self.setup_config = self.parser.get_setup_config()
        self.files = self.parser.get_files()
        self.icons = self.parser.get_icons()
        self.components = self.parser.get_components()
        self.tasks = self.parser.get_tasks()
        self.code = self.parser.get_code()
        
        compression = self.setup_config.get('Compression', 'zip')
        self.compressor = Compressor(method=compression)
    
    def build(self, output_path: str = None) -> str:
        output_dir = self.setup_config.get('OutputDir', '.')
        if output_path:
            output_dir = os.path.dirname(output_path) or '.'
        elif not os.path.isabs(output_dir) and self.script_dir:
            output_dir = os.path.join(self.script_dir, output_dir)
        
        app_name = self.setup_config.get('AppName', 'My Program')
        app_version = self.setup_config.get('AppVersion', '1.0')
        
        if not output_path:
            output_path = os.path.join(output_dir, f'{app_name} Setup {app_version}.exe')
        
        os.makedirs(output_dir, exist_ok=True)
        
        with tempfile.TemporaryDirectory() as temp_dir:
            archive_path = os.path.join(temp_dir, 'files.zip')
            self._package_files(temp_dir, archive_path)
            
            self._create_setup_package(archive_path, output_path)
        
        output_base = os.path.dirname(output_path) or '.'
        output_name = os.path.basename(output_path)
        output_folder = os.path.join(output_base, os.path.splitext(output_name)[0])
        return output_folder
    
    def _package_files(self, temp_dir: str, archive_path: str) -> None:
        package_dir = os.path.join(temp_dir, 'package')
        os.makedirs(package_dir, exist_ok=True)
        
        for file_entry in self.files:
            source = file_entry.get('Source', '')
            if source and not os.path.isabs(source) and not os.path.exists(source) and self.script_dir:
                candidate = os.path.join(self.script_dir, source)
                if os.path.exists(candidate):
                    source = candidate
            
            dest_dir = file_entry.get('DestDir', '{app}')
            dest_name = file_entry.get('DestName', '')
            
            rel_dest = dest_dir.strip()
            if rel_dest.lower().startswith('{app}'):
                rel_dest = rel_dest[5:]
            rel_dest = rel_dest.lstrip('/\\').replace('\\', os.sep).replace('/', os.sep)
            target_base = os.path.join(package_dir, rel_dest) if rel_dest else package_dir
            
            if os.path.isfile(source):
                filename = dest_name if dest_name else os.path.basename(source)
                dest_path = os.path.join(target_base, filename)
                os.makedirs(os.path.dirname(dest_path), exist_ok=True)
                shutil.copy2(source, dest_path)
            elif os.path.isdir(source):
                target_dir = os.path.join(target_base, dest_name) if dest_name else target_base
                os.makedirs(target_dir, exist_ok=True)
                shutil.copytree(source, target_dir, dirs_exist_ok=True)
        
        with zipfile.ZipFile(archive_path, 'w', zipfile.ZIP_DEFLATED) as zf:
            for root, dirs, files in os.walk(package_dir):
                for file in files:
                    file_path = os.path.join(root, file)
                    arcname = os.path.relpath(file_path, package_dir)
                    zf.write(file_path, arcname)
    
    def _find_stub_dir(self) -> str:
        possible_paths = []
        meipass = getattr(sys, '_MEIPASS', None)
        if meipass:
            possible_paths.append(os.path.join(meipass, 'runtime', 'setup_stub'))
        if getattr(sys, 'frozen', False):
            exe_dir = os.path.dirname(sys.executable)
            possible_paths.append(os.path.join(exe_dir, 'runtime', 'setup_stub'))
            possible_paths.append(os.path.join(exe_dir, 'setup_stub'))
        possible_paths.extend([
            os.path.join(os.path.dirname(__file__), '..', 'runtime', 'setup_stub'),
            os.path.join(os.path.dirname(__file__), '..', '..', 'runtime', 'setup_stub'),
            os.path.join(os.getcwd(), 'runtime', 'setup_stub'),
        ])
        
        exe_name = 'setup_stub.exe'
        for p in possible_paths:
            exe_path = os.path.join(p, exe_name)
            if os.path.exists(exe_path):
                return p
        
        for p in possible_paths:
            if os.path.isdir(p):
                return p
        
        return ''
    
    def _create_setup_package(self, archive_path: str, output_path: str) -> None:
        stub_dir = self._find_stub_dir()
        if not stub_dir or not os.path.exists(stub_dir):
            raise RuntimeError(
                f'Setup stub not found. Looked in: {stub_dir}\n'
                'Please ensure setup_stub directory exists with setup_stub.exe and _internal folder.'
            )
        
        if os.path.dirname(output_path):
            os.makedirs(os.path.dirname(output_path), exist_ok=True)
        
        output_base = os.path.dirname(output_path) or '.'
        output_name = os.path.basename(output_path)
        output_folder = os.path.join(output_base, os.path.splitext(output_name)[0])
        
        if os.path.exists(output_folder):
            shutil.rmtree(output_folder)
        
        shutil.copytree(stub_dir, output_folder)
        
        main_exe = os.path.join(output_folder, 'setup_stub.exe')
        target_exe = os.path.join(output_folder, output_name)
        
        if os.path.exists(main_exe):
            if os.path.exists(target_exe):
                os.remove(target_exe)
            os.rename(main_exe, target_exe)
        
        with open(archive_path, 'rb') as f:
            archive_data = f.read()
        
        config_data = self._serialize_config()
        
        marker = b'TRAESETUP\x00'
        footer = marker
        footer += struct.pack('<I', len(config_data))
        footer += struct.pack('<I', len(archive_data))
        footer += config_data
        footer += archive_data
        
        with open(target_exe, 'ab') as f:
            f.write(footer)
    
    def _serialize_config(self) -> bytes:
        license_text = self.setup_config.get(
            'LicenseText',
            '最终用户许可协议\n\n'
            '本软件按"现状"提供，不提供任何形式的保证。\n'
            '使用风险由您自行承担。'
        )
        
        config = {
            'setup': {
                'AppName': self.setup_config.get('AppName', '我的程序'),
                'AppVersion': self.setup_config.get('AppVersion', '1.0'),
                'DefaultDirName': self.setup_config.get(
                    'DefaultDirName',
                    os.path.join(r'C:\Program Files', self.setup_config.get('AppName', '我的程序'))
                ),
                'LicenseText': license_text,
                'Publisher': self.setup_config.get('AppPublisher', 'Trae Installer'),
            },
            'files': self.files,
            'icons': self.icons,
            'components': self.components,
            'tasks': self.tasks,
        }
        return json.dumps(config, ensure_ascii=False, indent=2).encode('utf-8')
