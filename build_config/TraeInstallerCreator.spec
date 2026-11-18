# -*- mode: python ; coding: utf-8 -*-

import os

block_cipher = None

src_dir = os.path.abspath(os.path.join(SPECPATH, '..', 'src'))

a = Analysis(
    [os.path.join(src_dir, 'gui', 'main_window.py')],
    pathex=[src_dir],
    binaries=[],
    datas=[
        (os.path.join(src_dir, 'runtime', 'setup_stub'), 'runtime/setup_stub'),
    ],
    hiddenimports=[
        'tkinter',
        'tkinter.ttk',
        'tkinter.filedialog',
        'tkinter.messagebox',
        'tkinter.simpledialog',
        'win32com.client',
        'zipfile',
        'struct',
        'json',
        'shutil',
        'tempfile',
        'winreg',
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.zipfiles,
    a.datas,
    [],
    name='TraeInstallerCreator',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=False,
    disable_windowed_traceback=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    icon=None,
)
