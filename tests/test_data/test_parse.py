import os
import sys
import struct
import json

tests_data_dir = os.path.dirname(os.path.abspath(__file__))
src_dir = os.path.abspath(os.path.join(tests_data_dir, '..', '..', 'src'))
sys.path.insert(0, src_dir)

from core.setup_builder import SetupBuilder

builder = SetupBuilder()
builder.load_script(os.path.join(tests_data_dir, 'test_setup.iss'))

print('=== Parsed Setup Config ===')
for k, v in builder.setup_config.items():
    print(f'  {k}: {v}')

print('\n=== Parsed Files ===')
for f in builder.files:
    print(f'  {f}')

print('\n=== Parsed Icons ===')
for i in builder.icons:
    print(f'  {i}')

config = builder._serialize_config()
print(f'\n=== Serialized Config ({len(config)} bytes) ===')
print(config.decode('utf-8')[:500])
