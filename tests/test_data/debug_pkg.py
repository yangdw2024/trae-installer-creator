import os
import struct

tests_dir = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
exe_path = os.path.join(tests_dir, 'test_output', 'Test App Setup 1.0', 'Test App Setup 1.0.exe')

with open(exe_path, 'rb') as f:
    data = f.read()

print(f'File size: {len(data)} bytes')

marker = b'TRAESETUP\x00'
marker_len = len(marker)
pos = data.rfind(marker)
print(f'\nMarker position: {pos}')
print(f'Bytes at marker+{marker_len} to marker+{marker_len + 16}:')
print(f'  Hex: {data[pos+marker_len:pos+marker_len+16].hex()}')
print(f'  As text: {data[pos+marker_len:pos+marker_len+16]}')

# Read what we think are the sizes
config_size = struct.unpack('<I', data[pos+marker_len:pos+marker_len+4])[0]
archive_size = struct.unpack('<I', data[pos+marker_len+4:pos+marker_len+8])[0]
print(f'\nconfig_size (pos+{marker_len}): {config_size}')
print(f'archive_size (pos+{marker_len+4}): {archive_size}')

# Check if config size is reasonable
print(f'\nExpected config size: ~500 bytes (small JSON)')
print(f'Actual: {config_size} bytes - {"WRONG!" if config_size > 10000 else "maybe OK"}')

# Let's look at what's right after the marker
print(f'\nFirst 200 bytes after marker:')
chunk = data[pos+marker_len:pos+marker_len+200]
print(chunk)
print()
print(f'As hex: {chunk[:60].hex()}')
