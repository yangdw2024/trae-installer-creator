import os
import sys
import struct
import json
import zipfile
import tempfile
import shutil

tests_data_dir = os.path.dirname(os.path.abspath(__file__))
src_dir = os.path.abspath(os.path.join(tests_data_dir, '..', '..', 'src'))
sys.path.insert(0, src_dir)

from core.setup_builder import SetupBuilder

builder = SetupBuilder()
builder.load_script(os.path.join(tests_data_dir, 'test_setup.iss'))

config = builder._serialize_config()
print(f'Config size: {len(config)} bytes')
print(f'Config content (first 300 chars):')
print(config[:300].decode('utf-8'))

# Check what the builder's stub dir is
stub_dir = builder._find_stub_dir()
print(f'\nStub dir: {stub_dir}')
print(f'Stub exists: {os.path.exists(stub_dir)}')
if os.path.exists(stub_dir):
    for f in os.listdir(stub_dir):
        fp = os.path.join(stub_dir, f)
        if os.path.isfile(fp):
            print(f'  - {f} ({os.path.getsize(fp)} bytes)')
