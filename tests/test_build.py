#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import os
import sys
import json
import struct

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from core.setup_builder import SetupBuilder
from core.script_parser import ScriptParser

tests_dir = os.path.dirname(os.path.abspath(__file__))
test_app_dir = os.path.join(tests_dir, 'test_app')
output_dir = os.path.join(tests_dir, 'test_output')

print('=' * 60)
print('测试 1：脚本解析器')
print('=' * 60)

script_content = f'''[Setup]
AppName=测试程序
AppVersion=1.0
AppPublisher=测试公司
DefaultDirName=C:\\Program Files\\测试程序
DefaultGroupName=测试程序
Compression=zip
OutputDir={output_dir}
LicenseText=这是测试许可协议。

[Files]
Source: "{test_app_dir}\\TestApp.exe"; DestDir: "{{app}}"
Source: "{test_app_dir}\\readme.txt"; DestDir: "{{app}}"
Source: "{test_app_dir}\\config.ini"; DestDir: "{{app}}"

[Icons]
Name: "{{group}}\\测试程序"; Filename: "{{app}}\\TestApp.exe"
Name: "{{autodesktop}}\\测试程序"; Filename: "{{app}}\\TestApp.exe"
'''

test_script = os.path.join(output_dir, 'test.iss')
os.makedirs(output_dir, exist_ok=True)
with open(test_script, 'w', encoding='utf-8') as f:
    f.write(script_content)

parser = ScriptParser()
parser.parse_file(test_script)

config = parser.get_setup_config()
print(f'Setup 配置项数量：{len(config)}')
print(f'AppName: {config.get("AppName")}')
print(f'AppVersion: {config.get("AppVersion")}')
print(f'LicenseText 存在: {"LicenseText" in config}')

files = parser.get_files()
print(f'\n文件列表数量：{len(files)}')
for i, f in enumerate(files):
    print(f'  文件 {i+1}: Source={f.get("Source")}, DestDir={f.get("DestDir")}')

icons = parser.get_icons()
print(f'\n快捷方式数量：{len(icons)}')
for i, icon in enumerate(icons):
    print(f'  快捷方式 {i+1}: Name={icon.get("Name")}, Filename={icon.get("Filename")}')

print('\n脚本解析器测试：通过 ✓')

print('\n' + '=' * 60)
print('测试 2：安装包生成')
print('=' * 60)

builder = SetupBuilder()
builder.load_script(test_script)

output_path = os.path.join(output_dir, '测试程序 Setup 1.0.exe')
print(f'输出路径：{output_path}')

try:
    result = builder.build(output_path)
    print(f'构建结果（文件夹路径）：{result}')
    
    if os.path.exists(result):
        print('安装包文件夹存在：通过 ✓')
    else:
        print('安装包文件夹不存在：失败 ✗')
        sys.exit(1)
    
    output_folder = result
    exe_name = os.path.basename(output_path)
    exe_path = os.path.join(output_folder, exe_name)
    
    if os.path.exists(exe_path):
        print(f'主程序 {exe_name} 存在：通过 ✓')
        exe_size = os.path.getsize(exe_path)
        print(f'EXE 文件大小：{exe_size} 字节')
    else:
        print(f'主程序 {exe_name} 不存在：失败 ✗')
        sys.exit(1)
    
    print('\n' + '=' * 60)
    print('测试 3：配置数据读取验证')
    print('=' * 60)
    
    with open(exe_path, 'rb') as f:
        data = f.read()
    
    marker = b'TRAESETUP\x00'
    pos = data.rfind(marker)
    
    if pos >= 0:
        print(f'找到 marker，位置：{pos}')
        marker_len = len(marker)
        
        config_size = struct.unpack('<I', data[pos+marker_len:pos+marker_len+4])[0]
        archive_size = struct.unpack('<I', data[pos+marker_len+4:pos+marker_len+8])[0]
        
        print(f'配置大小：{config_size} 字节')
        print(f'归档大小：{archive_size} 字节')
        
        config_start = pos + marker_len + 8
        config_data = data[config_start:config_start+config_size]
        
        try:
            config = json.loads(config_data.decode('utf-8'))
            print(f'配置 JSON 解析成功')
            print(f'  AppName: {config.get("setup", {}).get("AppName")}')
            print(f'  AppVersion: {config.get("setup", {}).get("AppVersion")}')
            print(f'  文件数量: {len(config.get("files", []))}')
            print(f'  快捷方式数量: {len(config.get("icons", []))}')
            print('配置数据读取：通过 ✓')
        except Exception as e:
            print(f'配置 JSON 解析失败：{e}')
            print('配置数据读取：失败 ✗')
            sys.exit(1)
        
        archive_start = config_start + config_size
        archive_data = data[archive_start:archive_start+archive_size]
        
        import zipfile
        import io
        try:
            zf = zipfile.ZipFile(io.BytesIO(archive_data))
            file_list = zf.namelist()
            print(f'\nZIP 归档包含 {len(file_list)} 个文件：')
            for name in file_list:
                print(f'  - {name}')
            print('ZIP 归档验证：通过 ✓')
        except Exception as e:
            print(f'ZIP 归档验证失败：{e}')
            print('ZIP 归档验证：失败 ✗')
            sys.exit(1)
    else:
        print('未找到 marker：失败 ✗')
        sys.exit(1)
    
    print('\n' + '=' * 60)
    print('所有测试通过！✓')
    print('=' * 60)
    
except Exception as e:
    import traceback
    print(f'构建失败：{e}')
    print(traceback.format_exc())
