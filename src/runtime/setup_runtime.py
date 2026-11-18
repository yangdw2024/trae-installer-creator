#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import os
import re
import sys
import struct
import json
import zipfile
import shutil
import tempfile
import winreg
import tkinter as tk
from tkinter import ttk, messagebox, filedialog


class SetupWizard:
    def __init__(self, root):
        self.root = root
        self.root.title('安装程序')
        self.root.geometry('600x450')
        self.root.resizable(False, False)
        
        self.config = {}
        self.archive_data = b''
        self.install_dir = ''
        self.current_page = 0
        self.installed_files = []
        
        self._load_config()
        self._create_widgets()
        self._show_page(0)
    
    def _load_config(self):
        if getattr(sys, 'frozen', False):
            exe_path = sys.executable
        else:
            exe_path = __file__
        
        try:
            with open(exe_path, 'rb') as f:
                data = f.read()
            
            marker = b'TRAESETUP\x00'
            pos = data.rfind(marker)
            if pos >= 0:
                marker_len = len(marker)
                config_size = struct.unpack('<I', data[pos+marker_len:pos+marker_len+4])[0]
                archive_size = struct.unpack('<I', data[pos+marker_len+4:pos+marker_len+8])[0]
                
                config_start = pos + marker_len + 8
                config_data = data[config_start:config_start+config_size]
                self.config = json.loads(config_data.decode('utf-8'))
                
                archive_start = config_start + config_size
                self.archive_data = data[archive_start:archive_start+archive_size]
        except Exception as e:
            print(f'Load config error: {e}')
        
        if not self.config:
            self.config = {
                'setup': {
                    'AppName': 'My Program',
                    'AppVersion': '1.0',
                    'DefaultDirName': r'C:\Program Files\My Program',
                    'LicenseText': 'This is a free software.\n\nNo warranty.'
                },
                'files': [],
                'icons': []
            }
    
    def _create_widgets(self):
        self.title_bar = tk.Frame(self.root, bg='#0078d7', height=60)
        self.title_bar.pack(fill=tk.X)
        self.title_bar.pack_propagate(False)
        
        self.title_label = tk.Label(self.title_bar, text='', bg='#0078d7', fg='white',
                                    font=('Microsoft YaHei', 14, 'bold'))
        self.title_label.pack(side=tk.LEFT, padx=20, pady=15)
        
        self.content_frame = tk.Frame(self.root)
        self.content_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=15)
        
        self.button_frame = tk.Frame(self.root, height=50)
        self.button_frame.pack(fill=tk.X, side=tk.BOTTOM)
        self.button_frame.pack_propagate(False)
        
        sep = ttk.Separator(self.button_frame, orient='horizontal')
        sep.pack(fill=tk.X)
        
        self.back_btn = ttk.Button(self.button_frame, text='< 上一步', command=self._prev_page)
        self.back_btn.pack(side=tk.LEFT, padx=10, pady=10)
        
        self.next_btn = ttk.Button(self.button_frame, text='下一步 >', command=self._next_page)
        self.next_btn.pack(side=tk.RIGHT, padx=10, pady=10)
        
        self.cancel_btn = ttk.Button(self.button_frame, text='取消', command=self._cancel)
        self.cancel_btn.pack(side=tk.RIGHT, padx=5, pady=10)
        
        self.pages = []
        self._create_welcome_page()
        self._create_license_page()
        self._create_dir_page()
        self._create_install_page()
        self._create_finish_page()
    
    def _create_welcome_page(self):
        page = tk.Frame(self.content_frame)
        
        app_name = self.config.get('setup', {}).get('AppName', '我的程序')
        app_version = self.config.get('setup', {}).get('AppVersion', '1.0')
        
        tk.Label(page, text=f'欢迎使用 {app_name} 安装程序', 
                font=('Microsoft YaHei', 14, 'bold')).pack(anchor='w', pady=(10, 5))
        tk.Label(page, text=f'版本 {app_version}', 
                font=('Microsoft YaHei', 10), fg='gray').pack(anchor='w', pady=(0, 20))
        
        tk.Label(page, text=f'这将在您的计算机上安装 {app_name}。',
                wraplength=540, justify='left').pack(anchor='w', pady=5)
        tk.Label(page, text='点击"下一步"继续，或点击"取消"退出安装程序。',
                wraplength=540, justify='left').pack(anchor='w', pady=5)
        
        self.pages.append(('欢迎', page))
    
    def _create_license_page(self):
        page = tk.Frame(self.content_frame)
        
        tk.Label(page, text='许可协议', 
                font=('Microsoft YaHei', 12, 'bold')).pack(anchor='w', pady=(5, 10))
        
        text_frame = tk.Frame(page)
        text_frame.pack(fill=tk.BOTH, expand=True)
        
        self.license_text = tk.Text(text_frame, wrap=tk.WORD, font=('Consolas', 9), height=10)
        scrollbar = ttk.Scrollbar(text_frame, orient=tk.VERTICAL, command=self.license_text.yview)
        self.license_text.configure(yscroll=scrollbar.set)
        
        self.license_text.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)
        
        license_text = self.config.get('setup', {}).get('LicenseText', 
            '最终用户许可协议\n\n'
            '本软件按"现状"提供，不提供任何形式的保证。\n'
            '使用风险由您自行承担。')
        self.license_text.insert(tk.END, license_text)
        self.license_text.config(state=tk.DISABLED)
        
        self.license_var = tk.StringVar(value='decline')
        radio_frame = tk.Frame(page)
        radio_frame.pack(fill=tk.X, pady=10)
        
        ttk.Radiobutton(radio_frame, text='我接受协议', 
                       variable=self.license_var, value='accept',
                       command=self._update_license_btn).pack(anchor='w', pady=2)
        ttk.Radiobutton(radio_frame, text='我不接受协议',
                       variable=self.license_var, value='decline',
                       command=self._update_license_btn).pack(anchor='w', pady=2)
        
        self.pages.append(('许可协议', page))
    
    def _update_license_btn(self):
        if self.license_var.get() == 'accept':
            self.next_btn.config(state=tk.NORMAL)
        else:
            self.next_btn.config(state=tk.DISABLED)
    
    def _create_dir_page(self):
        page = tk.Frame(self.content_frame)
        
        tk.Label(page, text='选择安装目录', 
                font=('Microsoft YaHei', 12, 'bold')).pack(anchor='w', pady=(5, 10))
        
        tk.Label(page, text='安装程序将把程序安装到以下文件夹。',
                wraplength=540, justify='left').pack(anchor='w', pady=(0, 10))
        
        dir_frame = tk.Frame(page)
        dir_frame.pack(fill=tk.X, pady=5)
        
        raw_default_dir = self.config.get('setup', {}).get('DefaultDirName', 
            r'C:\Program Files\我的程序')
        default_dir = self._expand_constants(raw_default_dir)
        
        self.dir_var = tk.StringVar(value=default_dir)
        ttk.Entry(dir_frame, textvariable=self.dir_var).pack(side=tk.LEFT, fill=tk.X, expand=True, padx=(0, 10))
        ttk.Button(dir_frame, text='浏览...', command=self._browse_dir).pack(side=tk.RIGHT)
        
        tk.Label(page, text='要继续，请点击"下一步"。您可以输入新路径或点击"浏览"来选择其他文件夹。',
                wraplength=540, justify='left', fg='gray').pack(anchor='w', pady=10)
        
        self.space_label = tk.Label(page, text='', fg='gray')
        self.space_label.pack(anchor='w', pady=5)
        self._check_disk_space()
        
        self.pages.append(('选择目录', page))
    
    def _browse_dir(self):
        dir_path = filedialog.askdirectory(title='选择安装目录')
        if dir_path:
            self.dir_var.set(dir_path)
            self._check_disk_space()
    
    def _check_disk_space(self):
        try:
            drive = os.path.splitdrive(self.dir_var.get())[0] + '\\'
            if os.path.exists(drive):
                import ctypes
                free_bytes = ctypes.c_ulonglong(0)
                ctypes.windll.kernel32.GetDiskFreeSpaceExW(
                    ctypes.c_wchar_p(drive), None, None, ctypes.pointer(free_bytes))
                free_mb = free_bytes.value / (1024 * 1024)
                self.space_label.config(text=f'所选驱动器可用空间：{free_mb:.0f} MB')
        except:
            pass
    
    def _create_install_page(self):
        page = tk.Frame(self.content_frame)
        
        tk.Label(page, text='正在安装...', 
                font=('Microsoft YaHei', 12, 'bold')).pack(anchor='w', pady=(5, 10))
        
        self.progress_var = tk.DoubleVar(value=0)
        self.progress_bar = ttk.Progressbar(page, variable=self.progress_var, maximum=100)
        self.progress_bar.pack(fill=tk.X, pady=10)
        
        self.status_label = tk.Label(page, text='正在准备安装...', fg='gray')
        self.status_label.pack(anchor='w', pady=5)
        
        self.detail_text = tk.Text(page, height=8, font=('Consolas', 8))
        self.detail_text.pack(fill=tk.BOTH, expand=True, pady=5)
        self.detail_text.config(state=tk.DISABLED)
        
        self.pages.append(('安装中', page))
    
    def _create_finish_page(self):
        page = tk.Frame(self.content_frame)
        
        app_name = self.config.get('setup', {}).get('AppName', '我的程序')
        
        tk.Label(page, text=f'{app_name} 安装完成！', 
                font=('Microsoft YaHei', 13, 'bold')).pack(anchor='w', pady=(20, 10))
        
        tk.Label(page, text='点击"完成"关闭安装程序。',
                wraplength=540, justify='left').pack(anchor='w', pady=5)
        
        self.create_desktop_var = tk.BooleanVar(value=True)
        ttk.Checkbutton(page, text='创建桌面快捷方式', 
                       variable=self.create_desktop_var).pack(anchor='w', pady=10)
        
        self.launch_var = tk.BooleanVar(value=False)
        ttk.Checkbutton(page, text=f'完成后启动 {app_name}',
                       variable=self.launch_var).pack(anchor='w', pady=5)
        
        self.pages.append(('完成', page))
    
    def _show_page(self, index):
        for i, (name, page) in enumerate(self.pages):
            page.pack_forget()
        
        self.current_page = index
        name, page = self.pages[index]
        page.pack(fill=tk.BOTH, expand=True)
        
        app_name = self.config.get('setup', {}).get('AppName', '安装程序')
        self.title_label.config(text=f'{app_name} 安装 - {name}')
        
        if index == 0:
            self.back_btn.config(state=tk.DISABLED)
            self.next_btn.config(state=tk.NORMAL, text='下一步 >')
        elif index == len(self.pages) - 1:
            self.back_btn.config(state=tk.DISABLED)
            self.next_btn.config(state=tk.NORMAL, text='完成')
            self.cancel_btn.config(state=tk.DISABLED)
        elif index == 1:
            self.back_btn.config(state=tk.NORMAL)
            self._update_license_btn()
            self.next_btn.config(text='下一步 >')
        elif index == 3:
            self.back_btn.config(state=tk.DISABLED)
            self.next_btn.config(state=tk.DISABLED, text='下一步 >')
        else:
            self.back_btn.config(state=tk.NORMAL)
            self.next_btn.config(state=tk.NORMAL, text='下一步 >')
    
    def _prev_page(self):
        if self.current_page > 0:
            self._show_page(self.current_page - 1)
    
    def _next_page(self):
        if self.current_page == len(self.pages) - 1:
            self._finish()
            return
        
        if self.current_page == 2:
            self._start_install()
            return
        
        self._show_page(self.current_page + 1)
    
    def _start_install(self):
        self.install_dir = self.dir_var.get()
        self._show_page(3)
        
        import threading
        threading.Thread(target=self._do_install, daemon=True).start()
    
    def _do_install(self):
        try:
            os.makedirs(self.install_dir, exist_ok=True)
            
            with tempfile.NamedTemporaryFile(delete=False, suffix='.zip') as temp_file:
                temp_file.write(self.archive_data)
                temp_zip = temp_file.name
            
            with zipfile.ZipFile(temp_zip, 'r') as zf:
                file_list = zf.namelist()
                total = len(file_list)
                
                for i, file_path in enumerate(file_list):
                    extract_path = os.path.join(self.install_dir, file_path)
                    if not file_path.endswith('/'):
                        os.makedirs(os.path.dirname(extract_path), exist_ok=True)
                        zf.extract(file_path, self.install_dir)
                        self.installed_files.append(file_path)
                    
                    progress = (i + 1) / total * 100
                    self.root.after(0, lambda p=progress, f=file_path: self._update_progress(p, f))
            
            os.unlink(temp_zip)
            
            self._create_icons()
            self._create_uninstaller()
            self._register_uninstaller()
            
            self.root.after(0, self._install_complete)
            
        except Exception as e:
            self.root.after(0, lambda: self._install_error(str(e)))
    
    def _update_progress(self, value, file_name):
        self.progress_var.set(value)
        self.status_label.config(text=f'正在安装：{file_name}')
        
        self.detail_text.config(state=tk.NORMAL)
        self.detail_text.insert(tk.END, f'  正在解压：{file_name}\n')
        self.detail_text.see(tk.END)
        self.detail_text.config(state=tk.DISABLED)
    
    def _expand_constants(self, path_str: str, normalize_path: bool = True) -> str:
        if not path_str:
            return path_str
        replacements = {
            '{autopf}': os.environ.get('ProgramFiles', r'C:\Program Files'),
            '{pf}': os.environ.get('ProgramFiles', r'C:\Program Files'),
            '{commonpf}': os.environ.get('ProgramFiles', r'C:\Program Files'),
            '{pf32}': os.environ.get('ProgramFiles(x86)', r'C:\Program Files (x86)'),
            '{commonpf32}': os.environ.get('ProgramFiles(x86)', r'C:\Program Files (x86)'),
            '{localappdata}': os.environ.get('LOCALAPPDATA', os.path.expanduser('~')),
            '{userappdata}': os.environ.get('LOCALAPPDATA', os.path.expanduser('~')),
            '{commonappdata}': os.environ.get('ProgramData', r'C:\ProgramData'),
            '{tmp}': tempfile.gettempdir(),
        }
        if self.install_dir:
            replacements['{app}'] = self.install_dir
        result = path_str
        matched = False
        for key, val in replacements.items():
            if key in result.lower():
                matched = True
                result = re.sub(re.escape(key), lambda _: val, result, flags=re.IGNORECASE)
        if normalize_path and matched:
            return os.path.normpath(result)
        return result

    def _find_primary_exe(self) -> str:
        files = self.config.get('files', [])
        for f_entry in files:
            src = f_entry.get('Source', '')
            if src.lower().endswith('.exe'):
                dest_dir = f_entry.get('DestDir', '{app}').strip()
                rel_dest = dest_dir[5:] if dest_dir.lower().startswith('{app}') else dest_dir
                rel_dest = rel_dest.lstrip('/\\')
                exe_name = f_entry.get('DestName') or os.path.basename(src)
                candidate = os.path.join(self.install_dir, rel_dest, exe_name) if rel_dest else os.path.join(self.install_dir, exe_name)
                if os.path.exists(candidate):
                    return candidate
        for rel_file in self.installed_files:
            if rel_file.lower().endswith('.exe') and not os.path.basename(rel_file).lower().startswith('unins'):
                candidate = os.path.join(self.install_dir, rel_file)
                if os.path.exists(candidate):
                    return candidate
        return ''

    def _get_desktop_dir(self) -> str:
        desktop_paths = [
            os.path.join(os.environ.get('USERPROFILE', ''), 'Desktop'),
            os.path.join(os.environ.get('USERPROFILE', ''), 'OneDrive', 'Desktop'),
            os.path.join(os.environ.get('PUBLIC', ''), 'Desktop'),
        ]
        for dp in desktop_paths:
            if dp and os.path.exists(dp):
                return dp
        return desktop_paths[0]

    def _create_icons(self):
        app_name = self.config.get('setup', {}).get('AppName', '我的程序')
        icons = self.config.get('icons', [])
        
        try:
            import win32com.client
            shell = win32com.client.Dispatch('WScript.Shell')
            
            for icon_entry in icons:
                name = icon_entry.get('Name', '')
                if '{autodesktop}' in name.lower() or '{userdesktop}' in name.lower():
                    continue
                
                filename = icon_entry.get('Filename', '')
                target = self._expand_constants(filename)
                parameters = self._expand_constants(icon_entry.get('Parameters', ''), normalize_path=False)
                working_dir = self._expand_constants(icon_entry.get('WorkingDir', ''))
                icon_file = self._expand_constants(icon_entry.get('IconFile', ''))
                
                if '{group}' in name.lower():
                    programs_dir = os.path.join(
                        os.environ.get('APPDATA', ''),
                        'Microsoft', 'Windows', 'Start Menu', 'Programs'
                    )
                    group_folder = os.path.join(programs_dir, app_name)
                    os.makedirs(group_folder, exist_ok=True)
                    clean_name = re.sub(r'\{group\}[\\/]?', '', name, flags=re.IGNORECASE)
                    shortcut_name = os.path.basename(clean_name) or app_name
                    shortcut_path = os.path.join(group_folder, shortcut_name + '.lnk')
                    self._make_shortcut(shell, target, shortcut_path, parameters, working_dir, icon_file)
                    
                    self.root.after(0, self._append_detail, '  已创建开始菜单快捷方式\n')
        except Exception as e:
            print(f'创建快捷方式错误：{e}')
    
    def _create_desktop_shortcuts(self):
        app_name = self.config.get('setup', {}).get('AppName', '我的程序')
        icons = self.config.get('icons', [])
        desktop_dir = self._get_desktop_dir()
        
        try:
            import win32com.client
            shell = win32com.client.Dispatch('WScript.Shell')
            
            desktop_entries = [
                icon for icon in icons
                if '{autodesktop}' in icon.get('Name', '').lower()
                or '{userdesktop}' in icon.get('Name', '').lower()
            ]
            
            if desktop_entries:
                for icon_entry in desktop_entries:
                    name = icon_entry.get('Name', '')
                    filename = icon_entry.get('Filename', '')
                    target = self._expand_constants(filename)
                    parameters = self._expand_constants(icon_entry.get('Parameters', ''), normalize_path=False)
                    working_dir = self._expand_constants(icon_entry.get('WorkingDir', ''))
                    icon_file = self._expand_constants(icon_entry.get('IconFile', ''))
                    clean_name = re.sub(r'\{(?:auto|user)desktop\}[\\/]?', '', name, flags=re.IGNORECASE)
                    shortcut_name = os.path.basename(clean_name) or app_name
                    shortcut_path = os.path.join(desktop_dir, shortcut_name + '.lnk')
                    self._make_shortcut(shell, target, shortcut_path, parameters, working_dir, icon_file)
            else:
                primary_exe = self._find_primary_exe()
                if primary_exe:
                    shortcut_path = os.path.join(desktop_dir, app_name + '.lnk')
                    self._make_shortcut(shell, primary_exe, shortcut_path)
        except Exception as e:
            print(f'创建桌面快捷方式错误：{e}')

    def _append_detail(self, text):
        self.detail_text.config(state=tk.NORMAL)
        self.detail_text.insert(tk.END, text)
        self.detail_text.see(tk.END)
        self.detail_text.config(state=tk.DISABLED)

    def _make_shortcut(self, shell, target, shortcut_path, parameters='', working_dir='', icon_file=''):
        try:
            shortcut = shell.CreateShortCut(shortcut_path)
            shortcut.Targetpath = target
            if parameters:
                shortcut.Arguments = parameters
            shortcut.WorkingDirectory = working_dir if working_dir else os.path.dirname(target)
            if icon_file:
                shortcut.IconLocation = icon_file
            shortcut.save()
        except Exception as e:
            print(f'Shortcut error: {e}')
    
    def _create_uninstaller(self):
        uninstall_dir = os.path.join(self.install_dir, 'uninstall')
        os.makedirs(uninstall_dir, exist_ok=True)
        
        uninstall_data = {
            'install_dir': self.install_dir,
            'installed_files': self.installed_files,
            'config': self.config.get('setup', {}),
            'icons': self.config.get('icons', [])
        }
        
        with open(os.path.join(uninstall_dir, 'uninstall_data.json'), 'w', encoding='utf-8') as f:
            json.dump(uninstall_data, f, ensure_ascii=False, indent=2)
        
        app_name = self.config.get('setup', {}).get('AppName', '我的程序')
        
        # 生成 PowerShell 卸载脚本（不依赖目标系统安装 Python）
        ps_script = rf'''# Uninstall Script for {app_name}
$ErrorActionPreference = 'SilentlyContinue'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$dataPath = Join-Path $scriptDir 'uninstall_data.json'

if (-not (Test-Path $dataPath)) {{
    [System.Windows.MessageBox]::Show('找不到卸载数据文件。', '卸载错误', 'OK', 'Error')
    exit 1
}}

$data = Get-Content $dataPath -Raw -Encoding UTF8 | ConvertFrom-Json
$installDir = $data.install_dir
$config = $data.config
$appName = $config.AppName

# 创建卸载确认窗口
Add-Type -AssemblyName System.Windows.Forms
$result = [System.Windows.Forms.MessageBox]::Show(
    "确定要卸载 $appName 吗？", 
    "卸载 $appName", 
    "YesNo", 
    "Question"
)

if ($result -ne [System.Windows.Forms.DialogResult]::Yes) {{
    exit 0
}}

# 创建进度窗口
$form = New-Object System.Windows.Forms.Form
$form.Text = "正在卸载 $appName"
$form.Size = New-Object System.Drawing.Size(400, 150)
$form.StartPosition = "CenterScreen"
$form.FormBorderStyle = "FixedDialog"
$form.MaximizeBox = $false

$label = New-Object System.Windows.Forms.Label
$label.Text = "正在卸载..."
$label.Location = New-Object System.Drawing.Point(20, 20)
$label.Size = New-Object System.Drawing.Size(350, 20)
$form.Controls.Add($label)

$progress = New-Object System.Windows.Forms.ProgressBar
$progress.Location = New-Object System.Drawing.Point(20, 50)
$progress.Size = New-Object System.Drawing.Size(350, 20)
$progress.Value = 0
$form.Controls.Add($progress)

$form.Show()
$form.Refresh()

# 删除文件
$label.Text = "正在删除文件..."
$progress.Value = 30
$form.Refresh()

Get-ChildItem -Path $installDir -Recurse -File | ForEach-Object {{
    try {{ Remove-Item $_.FullName -Force }} catch {{ }}
}}

# 删除目录
$label.Text = "正在删除目录..."
$progress.Value = 60
$form.Refresh()

Get-ChildItem -Path $installDir -Recurse -Directory | Sort-Object FullName -Descending | ForEach-Object {{
    try {{ Remove-Item $_.FullName -Force -Recurse }} catch {{ }}
}}

try {{ Remove-Item $installDir -Force -Recurse }} catch {{ }}

# 清理注册表
$label.Text = "正在清理注册表..."
$progress.Value = 90
$form.Refresh()

$regPaths = @(
    "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$appName",
    "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$appName"
)
foreach ($rp in $regPaths) {{
    if (Test-Path $rp) {{
        try {{ Remove-Item -Path $rp -Recurse -Force }} catch {{ }}
    }}
}}

# 删除开始菜单快捷方式
$startMenu = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
$appMenuFolder = Join-Path $startMenu $appName
if (Test-Path $appMenuFolder) {{
    try {{ Remove-Item $appMenuFolder -Recurse -Force }} catch {{ }}
}}

# 删除桌面快捷方式
$desktopPaths = @(
    (Join-Path $env:USERPROFILE "Desktop"),
    (Join-Path $env:USERPROFILE "OneDrive\Desktop"),
    (Join-Path $env:PUBLIC "Desktop")
)
foreach ($dp in $desktopPaths) {{
    $sc = Join-Path $dp "$appName.lnk"
    if (Test-Path $sc) {{
        try {{ Remove-Item $sc -Force }} catch {{ }}
    }}
}}

$progress.Value = 100
$form.Refresh()
Start-Sleep -Milliseconds 300
$form.Close()

[System.Windows.Forms.MessageBox]::Show("$appName 已成功卸载。", "卸载完成", "OK", "Information")
'''
        
        with open(os.path.join(uninstall_dir, 'uninstall.ps1'), 'w', encoding='utf-8') as f:
            f.write(ps_script)
        
        # 生成批处理文件调用 PowerShell
        uninstall_bat = os.path.join(uninstall_dir, 'uninstall.bat')
        with open(uninstall_bat, 'w', encoding='utf-8') as f:
            f.write('@echo off\n')
            f.write('cd /d "%~dp0"\n')
            f.write('powershell -ExecutionPolicy Bypass -File "uninstall.ps1"\n')
            f.write('if errorlevel 1 pause\n')
    
    def _register_uninstaller(self):
        app_name = self.config.get('setup', {}).get('AppName', '我的程序')
        app_version = self.config.get('setup', {}).get('AppVersion', '1.0')
        
        uninstall_exe = os.path.join(self.install_dir, 'uninstall', 'uninstall.bat')
        
        try:
            key_path = f'Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{app_name}'
            key = winreg.CreateKey(winreg.HKEY_CURRENT_USER, key_path)
            
            winreg.SetValueEx(key, 'DisplayName', 0, winreg.REG_SZ, app_name)
            winreg.SetValueEx(key, 'DisplayVersion', 0, winreg.REG_SZ, app_version)
            winreg.SetValueEx(key, 'UninstallString', 0, winreg.REG_SZ, f'"{uninstall_exe}"')
            winreg.SetValueEx(key, 'InstallLocation', 0, winreg.REG_SZ, self.install_dir)
            winreg.SetValueEx(key, 'Publisher', 0, winreg.REG_SZ, 'Trae Installer')
            
            winreg.CloseKey(key)
        except Exception as e:
            print(f'注册卸载程序错误：{e}')
    
    def _install_complete(self):
        self.progress_var.set(100)
        self.status_label.config(text='安装完成！')
        self._show_page(len(self.pages) - 1)
    
    def _install_error(self, error):
        messagebox.showerror('错误', f'安装失败：\n{error}')
        self.root.destroy()
    
    def _cancel(self):
        if messagebox.askyesno('取消安装', '确定要取消安装吗？'):
            self.root.destroy()
    
    def _finish(self):
        if self.create_desktop_var.get():
            self._create_desktop_shortcuts()
        
        if self.launch_var.get():
            first_exe = self._find_primary_exe()
            if first_exe and os.path.exists(first_exe):
                import subprocess
                subprocess.Popen(first_exe, cwd=self.install_dir)
        
        self.root.destroy()


def main():
    root = tk.Tk()
    app = SetupWizard(root)
    root.mainloop()


if __name__ == '__main__':
    main()
