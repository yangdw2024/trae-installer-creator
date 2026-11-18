import os
import struct

exe_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'test_output', '测试程序 Setup 1.0', '测试程序 Setup 1.0.exe')

with open(exe_path, 'rb') as f:
    data = f.read()

print(f'文件总大小：{len(data)} 字节')

marker = b'TRAESETUP\x00'
marker_len = len(marker)
pos = data.rfind(marker)
print(f'最后一个 marker 位置：{pos}')
print(f'marker 到文件末尾：{len(data) - pos} 字节')

# 检查 marker 前后的数据
if pos >= 0:
    print(f'\nmarker 前20字节：{data[pos-20:pos].hex()}')
    print(f'marker 后50字节：{data[pos:pos+50].hex()}')
    
    # 尝试读取 config_size 和 archive_size
    config_size = struct.unpack('<I', data[pos+marker_len:pos+marker_len+4])[0]
    archive_size = struct.unpack('<I', data[pos+marker_len+4:pos+marker_len+8])[0]
    print(f'\n读取的 config_size：{config_size}')
    print(f'读取的 archive_size：{archive_size}')
    print(f'两数之和 + {marker_len + 8}：{config_size + archive_size + marker_len + 8}')
    print(f'从 marker 到文件尾：{len(data) - pos}')
    
    # 找所有 marker 位置
    print('\n所有 marker 位置：')
    search_pos = 0
    count = 0
    while True:
        p = data.find(marker, search_pos)
        if p < 0:
            break
        print(f'  位置 {p}')
        count += 1
        search_pos = p + 1
    print(f'共找到 {count} 个 marker')
