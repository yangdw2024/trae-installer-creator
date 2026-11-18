import os
import sys
import struct
import json
import zipfile

tests_dir = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
exe_path = os.path.join(tests_dir, 'test_output', 'Test App Setup 1.0', 'Test App Setup 1.0.exe')

with open(exe_path, 'rb') as f:
    data = f.read()

marker = b'TRAESETUP\x00'
marker_len = len(marker)
pos = data.rfind(marker)
print(f'Marker position: {pos}')
print(f'Marker found: {pos >= 0}')

if pos >= 0:
    config_size = struct.unpack('<I', data[pos+marker_len:pos+marker_len+4])[0]
    archive_size = struct.unpack('<I', data[pos+marker_len+4:pos+marker_len+8])[0]
    print(f'Config size: {config_size}')
    print(f'Archive size: {archive_size}')
    
    config_start = pos + marker_len + 8
    config_data = data[config_start:config_start+config_size]
    config = json.loads(config_data.decode('utf-8'))
    print(f'\nConfig:')
    print(json.dumps(config, indent=2, ensure_ascii=False))
    
    archive_start = config_start + config_size
    archive_data = data[archive_start:archive_start+archive_size]
    
    import tempfile
    temp_zip = os.path.join(tempfile.gettempdir(), 'test_archive.zip')
    with open(temp_zip, 'wb') as f:
        f.write(archive_data)
    
    print(f'\nArchive saved to: {temp_zip}')
    
    with zipfile.ZipFile(temp_zip, 'r') as zf:
        print(f'\nFiles in archive:')
        for name in zf.namelist():
            print(f'  - {name}')
    
    os.unlink(temp_zip)
