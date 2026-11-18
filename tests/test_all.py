#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import os
import sys
import traceback

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

print('=' * 70)
print('全面检查所有模块')
print('=' * 70)

errors = []

def test_import(name, module_name=None):
    try:
        if module_name:
            if module_name.startswith('from ') or module_name.startswith('import '):
                exec(module_name)
            else:
                exec(f'import {module_name}')
        else:
            exec(f'import {name}')
        print(f'✓ {name} 导入成功')
        return True
    except Exception as e:
        errors.append(f'✗ {name} 导入失败: {e}')
        return False

print('\n[1] 标准库导入测试')
test_import('os')
test_import('sys')
test_import('struct')
test_import('json')
test_import('zipfile')
test_import('shutil')
test_import('tempfile')
test_import('threading')
test_import('traceback')
test_import('winreg')

print('\n[2] GUI库导入测试')
test_import('tkinter', 'tkinter as tk')
test_import('ttk', 'from tkinter import ttk')
test_import('filedialog', 'from tkinter import filedialog')
test_import('messagebox', 'from tkinter import messagebox')
test_import('simpledialog', 'from tkinter import simpledialog')

print('\n[3] 第三方库导入测试')
test_import('win32com', 'win32com.client')

print('\n[4] 项目模块导入测试')
try:
    from core.setup_builder import SetupBuilder
    print('✓ core.setup_builder 导入成功')
except Exception as e:
    errors.append(f'✗ core.setup_builder 导入失败: {e}')
    traceback.print_exc()

try:
    from core.script_parser import ScriptParser
    print('✓ core.script_parser 导入成功')
except Exception as e:
    errors.append(f'✗ core.script_parser 导入失败: {e}')
    traceback.print_exc()

try:
    from core.compressor import Compressor
    print('✓ core.compressor 导入成功')
except Exception as e:
    errors.append(f'✗ core.compressor 导入失败: {e}')
    traceback.print_exc()

print('\n' + '=' * 70)
print('检查文件和目录结构')
print('=' * 70)

src_dir = os.path.join(os.path.dirname(__file__), '..', 'src')

files_to_check = [
    ('主界面', os.path.join(src_dir, 'gui', 'main_window.py')),
    ('安装包生成器', os.path.join(src_dir, 'core', 'setup_builder.py')),
    ('脚本解析器', os.path.join(src_dir, 'core', 'script_parser.py')),
    ('压缩器', os.path.join(src_dir, 'core', 'compressor.py')),
    ('安装向导', os.path.join(src_dir, 'runtime', 'setup_runtime.py')),
    ('setup_stub.exe', os.path.join(src_dir, 'runtime', 'setup_stub', 'setup_stub.exe')),
    ('使用说明', os.path.join(os.path.dirname(__file__), '..', 'release', '使用说明.txt')),
]

for name, path in files_to_check:
    if os.path.exists(path):
        print(f'✓ {name}: {path}')
    else:
        errors.append(f'✗ {name} 不存在: {path}')

print('\n' + '=' * 70)
print('检查 setup_stub 目录完整性')
print('=' * 70)

stub_dir = os.path.join(src_dir, 'runtime', 'setup_stub')
if os.path.exists(stub_dir):
    dlls = [f for f in os.listdir(stub_dir) if f.endswith('.dll')]
    print(f'DLL文件 ({len(dlls)}个): {dlls}')
    
    if os.path.exists(os.path.join(stub_dir, '_internal')):
        print('✓ _internal 目录存在')
        internal_files = os.listdir(os.path.join(stub_dir, '_internal'))
        print(f'  _internal 内容: {internal_files}')
    else:
        errors.append('✗ _internal 目录不存在')
    
    if os.path.exists(os.path.join(stub_dir, 'setup_stub.exe')):
        exe_size = os.path.getsize(os.path.join(stub_dir, 'setup_stub.exe'))
        print(f'✓ setup_stub.exe 存在 ({exe_size} 字节)')
    else:
        errors.append('✗ setup_stub.exe 不存在')
else:
    errors.append('✗ setup_stub 目录不存在')

print('\n' + '=' * 70)
if errors:
    print('发现错误:')
    for e in errors:
        print(f'  {e}')
    print(f'\n共发现 {len(errors)} 个错误')
else:
    print('所有检查通过！没有发现错误。')
print('=' * 70)
