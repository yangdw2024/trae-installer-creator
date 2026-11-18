#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import os
import sys
import json
import struct
import zipfile
import tempfile

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

print('=' * 60)
print('测试安装向导配置读取逻辑')
print('=' * 60)

exe_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'test_output', '测试程序 Setup 1.0', '测试程序 Setup 1.0.exe')

with open(exe_path, 'rb') as f:
    data = f.read()

marker = b'TRAESETUP\x00'
marker_len = len(marker)
pos = data.rfind(marker)

if pos >= 0:
    config_size = struct.unpack('<I', data[pos+marker_len:pos+marker_len+4])[0]
    archive_size = struct.unpack('<I', data[pos+marker_len+4:pos+marker_len+8])[0]
    
    config_start = pos + marker_len + 8
    config_data = data[config_start:config_start+config_size]
    
    config = json.loads(config_data.decode('utf-8'))
    print(f'✓ 配置 JSON 解析成功')
    print(f'  AppName: {config["setup"]["AppName"]}')
    print(f'  AppVersion: {config["setup"]["AppVersion"]}')
    print(f'  DefaultDirName: {config["setup"]["DefaultDirName"]}')
    print(f'  LicenseText 长度: {len(config["setup"]["LicenseText"])}')
    print(f'  文件数量: {len(config["files"])}')
    print(f'  快捷方式数量: {len(config["icons"])}')
    
    archive_start = config_start + config_size
    archive_data = data[archive_start:archive_start+archive_size]
    
    with tempfile.NamedTemporaryFile(delete=False, suffix='.zip') as temp_file:
        temp_file.write(archive_data)
        temp_zip = temp_file.name
    
    with zipfile.ZipFile(temp_zip, 'r') as zf:
        file_list = zf.namelist()
        print(f'\n✓ ZIP 归档包含 {len(file_list)} 个文件：')
        for name in file_list:
            print(f'  - {name}')
    
    os.unlink(temp_zip)
    
    print('\n' + '=' * 60)
    print('安装向导数据读取测试：通过 ✓')
    print('=' * 60)
else:
    print('未找到 marker，测试失败')
