#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import os
import sys
import threading
import traceback

try:
    import tkinter as tk
    from tkinter import ttk, filedialog, messagebox, simpledialog
except ImportError as e:
    try:
        import ctypes
        ctypes.windll.user32.MessageBoxW(0, 
            f"Failed to load tkinter GUI library.\n\nError: {str(e)}\n\n"
            "Please install Python with tkinter support, or use the pre-built EXE version.",
            "Trae Installer Creator - Startup Error", 0x10)
    except:
        print(f"Fatal Error: Failed to load tkinter: {str(e)}")
    sys.exit(1)

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

try:
    from core.setup_builder import SetupBuilder
    from core.script_parser import ScriptParser
except ImportError as e:
    messagebox.showerror("Import Error", f"Failed to import core modules: {str(e)}")
    sys.exit(1)


class SetupCreatorGUI:
    def __init__(self, root):
        self.root = root
        self.root.title('Trae 安装包制作工具')
        self.root.geometry('1000x700')
        self.root.minsize(800, 600)
        
        self.builder = SetupBuilder()
        self.current_script = ''
        self.license_text_content = self._default_license()
        
        self._create_widgets()
    
    def _default_license(self):
        return (
            "最终用户许可协议\n"
            "==========================\n\n"
            "重要 - 请仔细阅读：\n\n"
            "本最终用户许可协议（以下简称\"协议\"）是您（个人或单一实体）"
            "与软件作者之间就上述软件产品达成的法律协议。\n\n"
            "安装、复制或以其他方式使用本软件，即表示您同意受本协议条款的约束。"
            "如果您不同意本协议的条款，请不要安装或使用本软件。\n\n"
            "1. 许可授予\n"
            "   您可以在单台计算机上安装和使用本软件的一份副本。\n\n"
            "2. 其他权利和限制说明\n"
            "   - 不得对本软件进行反向工程、反编译或反汇编。\n"
            "   - 本软件按\"现状\"提供，不提供任何形式的保证。\n\n"
            "3. 版权\n"
            "   本软件的所有所有权和版权均归软件作者所有。\n\n"
            "4. 无担保\n"
            "   本软件按\"现状\"提供，不提供任何明示或暗示的担保。\n\n"
            "5. 责任限制\n"
            "   在任何情况下，作者均不对因使用本软件而产生的任何损害承担责任。\n"
        )
    
    def _create_widgets(self):
        main_frame = ttk.Frame(self.root, padding=10)
        main_frame.pack(fill=tk.BOTH, expand=True)
        
        banner = tk.Frame(main_frame, bg='#0078d7', height=50)
        banner.pack(fill=tk.X)
        banner.pack_propagate(False)
        
        tk.Label(banner, text='Trae 安装包制作工具', 
                bg='#0078d7', fg='white', font=('Microsoft YaHei', 14, 'bold')
                ).pack(side=tk.LEFT, padx=20, pady=10)
        
        tk.Label(banner, text='轻松制作专业的安装程序',
                bg='#0078d7', fg='white', font=('Microsoft YaHei', 9)
                ).pack(side=tk.RIGHT, padx=20, pady=15)
        
        ttk.Frame(main_frame, height=5).pack()
        
        quick_bar = tk.Frame(main_frame, bg='#f0f0f0', height=70)
        quick_bar.pack(fill=tk.X)
        quick_bar.pack_propagate(False)
        
        tk.Label(quick_bar, text='快捷操作：', 
                bg='#f0f0f0', font=('Microsoft YaHei', 10, 'bold')
                ).pack(side=tk.LEFT, padx=15, pady=20)
        
        ttk.Button(quick_bar, text='  一键生成安装包  ', 
                  command=self._quick_build, style='Accent.TButton'
                  ).pack(side=tk.LEFT, padx=10, pady=15)
        
        ttk.Button(quick_bar, text='  智能添加软件文件夹  ', 
                  command=self._add_program_folder
                  ).pack(side=tk.LEFT, padx=10, pady=15)
        
        ttk.Button(quick_bar, text='  加载模板  ', 
                  command=self._load_template
                  ).pack(side=tk.LEFT, padx=10, pady=15)
        
        ttk.Frame(main_frame, height=5).pack()
        
        self.notebook = ttk.Notebook(main_frame)
        self.notebook.pack(fill=tk.BOTH, expand=True)
        
        self._create_setup_tab(self.notebook)
        self._create_files_tab(self.notebook)
        self._create_license_tab(self.notebook)
        self._create_icons_tab(self.notebook)
        self._create_script_tab(self.notebook)
        
        self.script_tab_index = self.notebook.index('end') - 1
        self._last_tab_index = 0
        self.notebook.bind('<<NotebookTabChanged>>', self._on_tab_changed)
        
        button_frame = ttk.Frame(main_frame, padding=5)
        button_frame.pack(fill=tk.X, anchor=tk.E)
        
        ttk.Button(button_frame, text='打开脚本', command=self._open_script).pack(side=tk.LEFT, padx=5)
        ttk.Button(button_frame, text='保存脚本', command=self._save_script).pack(side=tk.LEFT, padx=5)
        
        ttk.Button(button_frame, text='生成安装包', 
                  command=self._build_setup, width=25).pack(side=tk.RIGHT, padx=5)
        
        self.status_bar = ttk.Label(main_frame, text='就绪', relief=tk.SUNKEN)
        self.status_bar.pack(fill=tk.X)
    
    def _create_setup_tab(self, notebook):
        frame = ttk.Frame(notebook, padding=15)
        notebook.add(frame, text='基本设置')
        
        grid_row = 0
        
        ttk.Separator(frame, orient='horizontal').grid(row=grid_row, column=0, columnspan=3, sticky='ew', pady=5)
        grid_row += 1
        
        ttk.Label(frame, text='应用程序信息', font=('Microsoft YaHei', 10, 'bold')
                 ).grid(row=grid_row, column=0, sticky=tk.W, padx=5, pady=5)
        grid_row += 1
        
        ttk.Label(frame, text='软件名称：').grid(row=grid_row, column=0, sticky=tk.W, padx=5, pady=5)
        self.app_name_var = tk.StringVar(value='我的程序')
        ttk.Entry(frame, textvariable=self.app_name_var, width=50).grid(row=grid_row, column=1, padx=5, pady=5, sticky=tk.W)
        grid_row += 1
        
        ttk.Label(frame, text='软件版本：').grid(row=grid_row, column=0, sticky=tk.W, padx=5, pady=5)
        self.app_version_var = tk.StringVar(value='1.0')
        ttk.Entry(frame, textvariable=self.app_version_var, width=20).grid(row=grid_row, column=1, padx=5, pady=5, sticky=tk.W)
        grid_row += 1
        
        ttk.Label(frame, text='发布者：').grid(row=grid_row, column=0, sticky=tk.W, padx=5, pady=5)
        self.publisher_var = tk.StringVar(value='我的公司')
        ttk.Entry(frame, textvariable=self.publisher_var, width=50).grid(row=grid_row, column=1, padx=5, pady=5, sticky=tk.W)
        grid_row += 1
        
        ttk.Label(frame, text='网站：').grid(row=grid_row, column=0, sticky=tk.W, padx=5, pady=5)
        self.website_var = tk.StringVar(value='')
        ttk.Entry(frame, textvariable=self.website_var, width=50).grid(row=grid_row, column=1, padx=5, pady=5, sticky=tk.W)
        grid_row += 1
        
        ttk.Separator(frame, orient='horizontal').grid(row=grid_row, column=0, columnspan=3, sticky='ew', pady=10)
        grid_row += 1
        
        ttk.Label(frame, text='安装设置', font=('Microsoft YaHei', 10, 'bold')
                 ).grid(row=grid_row, column=0, sticky=tk.W, padx=5, pady=5)
        grid_row += 1
        
        ttk.Label(frame, text='默认安装目录：').grid(row=grid_row, column=0, sticky=tk.W, padx=5, pady=5)
        self.default_dir_var = tk.StringVar(value=r'C:\Program Files\我的程序')
        ttk.Entry(frame, textvariable=self.default_dir_var, width=50).grid(row=grid_row, column=1, padx=5, pady=5, sticky=tk.W)
        grid_row += 1
        
        ttk.Label(frame, text='开始菜单组：').grid(row=grid_row, column=0, sticky=tk.W, padx=5, pady=5)
        self.default_group_var = tk.StringVar(value='我的程序')
        ttk.Entry(frame, textvariable=self.default_group_var, width=50).grid(row=grid_row, column=1, padx=5, pady=5, sticky=tk.W)
        grid_row += 1
        
        ttk.Label(frame, text='卸载程序图标：').grid(row=grid_row, column=0, sticky=tk.W, padx=5, pady=5)
        self.uninstall_icon_var = tk.StringVar(value=r'{app}\MyProg.exe')
        ttk.Entry(frame, textvariable=self.uninstall_icon_var, width=50).grid(row=grid_row, column=1, padx=5, pady=5, sticky=tk.W)
        grid_row += 1
        
        ttk.Label(frame, text='压缩方式：').grid(row=grid_row, column=0, sticky=tk.W, padx=5, pady=5)
        self.compression_var = tk.StringVar(value='zip')
        compression_combo = ttk.Combobox(frame, textvariable=self.compression_var, 
                                         values=['zip', 'zlib', 'bzip2', 'lzma', 'lzma2'], width=17)
        compression_combo.grid(row=grid_row, column=1, padx=5, pady=5, sticky=tk.W)
        grid_row += 1
        
        ttk.Separator(frame, orient='horizontal').grid(row=grid_row, column=0, columnspan=3, sticky='ew', pady=10)
        grid_row += 1
        
        ttk.Label(frame, text='输出设置', font=('Microsoft YaHei', 10, 'bold')
                 ).grid(row=grid_row, column=0, sticky=tk.W, padx=5, pady=5)
        grid_row += 1
        
        ttk.Label(frame, text='输出目录：').grid(row=grid_row, column=0, sticky=tk.W, padx=5, pady=5)
        self.output_dir_var = tk.StringVar(value=os.path.join(os.path.expanduser('~'), 'Desktop'))
        ttk.Entry(frame, textvariable=self.output_dir_var, width=50).grid(row=grid_row, column=1, padx=5, pady=5, sticky=tk.W)
        ttk.Button(frame, text='浏览', command=self._browse_output_dir).grid(row=grid_row, column=2, padx=5, pady=5)
        grid_row += 1
        
        ttk.Label(frame, text='输出文件名：').grid(row=grid_row, column=0, sticky=tk.W, padx=5, pady=5)
        self.output_name_var = tk.StringVar(value='')
        ttk.Entry(frame, textvariable=self.output_name_var, width=50).grid(row=grid_row, column=1, padx=5, pady=5, sticky=tk.W)
        ttk.Label(frame, text='(留空自动命名)').grid(row=grid_row, column=2, padx=5, pady=5)
    
    def _create_files_tab(self, notebook):
        frame = ttk.Frame(notebook, padding=10)
        notebook.add(frame, text='文件管理')
        
        info_frame = tk.Frame(frame, bg='#e8f5e9', padx=10, pady=8)
        info_frame.pack(fill=tk.X, pady=(0, 10))
        tk.Label(info_frame, text='提示：在这里添加您的软件文件，所有文件将被安装到用户电脑上。',
                bg='#e8f5e9', fg='#2e7d32', wraplength=900, justify='left').pack(anchor='w')
        
        file_frame = ttk.Frame(frame)
        file_frame.pack(fill=tk.BOTH, expand=True)
        
        columns = ('source', 'dest_dir', 'dest_name', 'flags')
        self.files_tree = ttk.Treeview(file_frame, columns=columns, show='headings')
        self.files_tree.heading('source', text='源文件/文件夹')
        self.files_tree.heading('dest_dir', text='安装目标目录')
        self.files_tree.heading('dest_name', text='安装后重命名')
        self.files_tree.heading('flags', text='标志')
        
        self.files_tree.column('source', width=350)
        self.files_tree.column('dest_dir', width=200)
        self.files_tree.column('dest_name', width=200)
        self.files_tree.column('flags', width=100)
        
        scrollbar = ttk.Scrollbar(file_frame, orient=tk.VERTICAL, command=self.files_tree.yview)
        self.files_tree.configure(yscroll=scrollbar.set)
        
        self.files_tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)
        
        button_frame = ttk.Frame(frame)
        button_frame.pack(fill=tk.X, padx=5, pady=10)
        
        ttk.Button(button_frame, text='添加文件', command=self._add_file).pack(side=tk.LEFT, padx=5)
        ttk.Button(button_frame, text='添加文件夹', command=self._add_directory).pack(side=tk.LEFT, padx=5)
        ttk.Button(button_frame, text='智能添加软件文件夹', command=self._add_program_folder).pack(side=tk.LEFT, padx=5)
        ttk.Button(button_frame, text='删除选中', command=self._remove_file).pack(side=tk.LEFT, padx=5)
        ttk.Button(button_frame, text='清空全部', command=self._clear_files).pack(side=tk.LEFT, padx=5)
    
    def _create_license_tab(self, notebook):
        frame = ttk.Frame(notebook, padding=10)
        notebook.add(frame, text='许可协议')
        
        info_frame = tk.Frame(frame, bg='#fff3e0', padx=10, pady=8)
        info_frame.pack(fill=tk.X, pady=(0, 10))
        tk.Label(info_frame, 
                text='提示：用户在安装过程中将看到此许可协议，必须接受才能继续安装。',
                bg='#fff3e0', fg='#e65100', wraplength=900, justify='left').pack(anchor='w')
        
        btn_frame = ttk.Frame(frame)
        btn_frame.pack(fill=tk.X, pady=5)
        
        ttk.Button(btn_frame, text='从文件加载协议', command=self._load_license_file).pack(side=tk.LEFT, padx=5)
        ttk.Button(btn_frame, text='保存协议到文件', command=self._save_license_file).pack(side=tk.LEFT, padx=5)
        ttk.Button(btn_frame, text='恢复默认', command=self._restore_license).pack(side=tk.LEFT, padx=5)
        
        ttk.Label(frame, text='协议内容：').pack(anchor=tk.W, pady=(10, 5))
        
        self.license_text_widget = tk.Text(frame, wrap=tk.WORD, font=('Consolas', 10), height=20)
        license_scroll = ttk.Scrollbar(frame, orient=tk.VERTICAL, command=self.license_text_widget.yview)
        self.license_text_widget.configure(yscroll=license_scroll.set)
        
        self.license_text_widget.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        license_scroll.pack(side=tk.RIGHT, fill=tk.Y)
        
        self.license_text_widget.insert(tk.END, self.license_text_content)
    
    def _create_icons_tab(self, notebook):
        frame = ttk.Frame(notebook, padding=10)
        notebook.add(frame, text='快捷方式')
        
        info_frame = tk.Frame(frame, bg='#e3f2fd', padx=10, pady=8)
        info_frame.pack(fill=tk.X, pady=(0, 10))
        tk.Label(info_frame, 
                text='提示：为开始菜单和桌面创建快捷方式。'
                     '{group} 表示开始菜单程序组，{autodesktop} 表示所有用户的桌面。',
                bg='#e3f2fd', fg='#0d47a1', wraplength=900, justify='left').pack(anchor='w')
        
        icon_frame = ttk.Frame(frame)
        icon_frame.pack(fill=tk.BOTH, expand=True)
        
        columns = ('name', 'filename', 'parameters', 'working_dir', 'icon_file')
        self.icons_tree = ttk.Treeview(icon_frame, columns=columns, show='headings')
        self.icons_tree.heading('name', text='快捷方式名称和位置')
        self.icons_tree.heading('filename', text='目标程序')
        self.icons_tree.heading('parameters', text='参数')
        self.icons_tree.heading('working_dir', text='工作目录')
        self.icons_tree.heading('icon_file', text='图标文件')
        
        self.icons_tree.column('name', width=280)
        self.icons_tree.column('filename', width=280)
        self.icons_tree.column('parameters', width=100)
        self.icons_tree.column('working_dir', width=150)
        self.icons_tree.column('icon_file', width=150)
        
        scrollbar = ttk.Scrollbar(icon_frame, orient=tk.VERTICAL, command=self.icons_tree.yview)
        self.icons_tree.configure(yscroll=scrollbar.set)
        
        self.icons_tree.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)
        
        button_frame = ttk.Frame(frame)
        button_frame.pack(fill=tk.X, padx=5, pady=10)
        
        ttk.Button(button_frame, text='添加开始菜单快捷方式', command=self._add_start_menu_icon).pack(side=tk.LEFT, padx=5)
        ttk.Button(button_frame, text='添加桌面快捷方式', command=self._add_desktop_icon).pack(side=tk.LEFT, padx=5)
        ttk.Button(button_frame, text='添加自定义快捷方式', command=self._add_icon).pack(side=tk.LEFT, padx=5)
        ttk.Button(button_frame, text='删除选中', command=self._remove_icon).pack(side=tk.LEFT, padx=5)
    
    def _create_script_tab(self, notebook):
        frame = ttk.Frame(notebook, padding=10)
        notebook.add(frame, text='脚本（高级）')
        
        info_frame = tk.Frame(frame, bg='#f3e5f5', padx=10, pady=8)
        info_frame.pack(fill=tk.X, pady=(0, 10))
        tk.Label(info_frame, 
                text='这里显示当前设置对应的安装脚本（.iss 格式）。'
                     '高级用户可以直接在这里手动修改，保存时更改会同步到其他标签页。',
                bg='#f3e5f5', fg='#4a148c', wraplength=900, justify='left').pack(anchor='w')
        
        self.script_text = tk.Text(frame, wrap=tk.NONE, font=('Consolas', 10))
        
        v_scroll = ttk.Scrollbar(frame, orient=tk.VERTICAL, command=self.script_text.yview)
        h_scroll = ttk.Scrollbar(frame, orient=tk.HORIZONTAL, command=self.script_text.xview)
        self.script_text.configure(yscroll=v_scroll.set, xscroll=h_scroll.set)
        
        self.script_text.pack(side=tk.TOP, fill=tk.BOTH, expand=True)
        v_scroll.pack(side=tk.RIGHT, fill=tk.Y)
        h_scroll.pack(side=tk.BOTTOM, fill=tk.X)
    
    def _quick_build(self):
        app_name = self.app_name_var.get().strip()
        if not app_name:
            messagebox.showwarning('提示', '请先输入软件名称！')
            return
        
        file_count = len(self.files_tree.get_children())
        if file_count == 0:
            answer = messagebox.askyesno('未添加文件', 
                '还没有添加任何文件。您想现在选择一个软件文件夹吗？')
            if answer:
                self._add_program_folder()
            return
        
        self._build_setup()
    
    def _add_program_folder(self):
        folder_path = filedialog.askdirectory(title='选择您的软件文件夹')
        if not folder_path:
            return
        
        for item in self.files_tree.get_children():
            self.files_tree.delete(item)
        
        self.files_tree.insert('', tk.END, values=(folder_path, '{app}', '', ''))
        
        folder_name = os.path.basename(folder_path)
        if not self.app_name_var.get() or self.app_name_var.get() == '我的程序':
            self.app_name_var.set(folder_name)
            self.default_group_var.set(folder_name)
            self.default_dir_var.set(r'C:\Program Files\\' + folder_name)
        
        main_exe = None
        try:
            for f in os.listdir(folder_path):
                if f.lower().endswith('.exe') and not f.lower().startswith('unins'):
                    main_exe = f
                    break
        except:
            pass
        
        if main_exe:
            app_name = self.app_name_var.get()
            self.icons_tree.insert('', tk.END, values=(
                '{group}\\' + app_name,
                '{app}\\' + main_exe,
                '',
                '',
                ''
            ))
            self.icons_tree.insert('', tk.END, values=(
                '{autodesktop}\\' + app_name,
                '{app}\\' + main_exe,
                '',
                '',
                ''
            ))
            self.uninstall_icon_var.set('{app}\\' + main_exe)
        
        messagebox.showinfo('完成', 
            f'文件夹添加成功！\n\n'
            f'文件夹：{folder_path}\n'
            f'找到主程序：{main_exe or "未找到"}\n\n'
            f'点击"一键生成安装包"按钮即可生成安装程序。')
    
    def _load_template(self):
        self.app_name_var.set('我的程序')
        self.app_version_var.set('1.0')
        self.publisher_var.set('我的公司')
        self.default_dir_var.set(r'C:\Program Files\我的程序')
        self.default_group_var.set('我的程序')
        self.compression_var.set('zip')
        self.license_text_widget.delete(1.0, tk.END)
        self.license_text_widget.insert(tk.END, self._default_license())
        
        messagebox.showinfo('模板已加载', '已加载默认模板。')
    
    def _browse_output_dir(self):
        dir_path = filedialog.askdirectory(initialdir=self.output_dir_var.get())
        if dir_path:
            self.output_dir_var.set(dir_path)
    
    def _add_file(self):
        file_path = filedialog.askopenfilename(title='选择要添加的文件')
        if file_path:
            self.files_tree.insert('', tk.END, values=(file_path, '{app}', '', ''))
    
    def _add_directory(self):
        dir_path = filedialog.askdirectory(title='选择要添加的文件夹')
        if dir_path:
            self.files_tree.insert('', tk.END, values=(dir_path, '{app}', '', ''))
    
    def _remove_file(self):
        selected = self.files_tree.selection()
        for item in selected:
            self.files_tree.delete(item)
    
    def _clear_files(self):
        if messagebox.askyesno('确认', '确定要清空所有文件吗？'):
            for item in self.files_tree.get_children():
                self.files_tree.delete(item)
    
    def _add_start_menu_icon(self):
        app_name = self.app_name_var.get()
        exe_path = filedialog.askopenfilename(
            title='选择目标程序',
            filetypes=[('可执行文件', '*.exe'), ('所有文件', '*.*')]
        )
        if exe_path:
            exe_name = os.path.basename(exe_path)
            name = simpledialog.askstring('快捷方式名称', 
                '请输入开始菜单快捷方式名称：', initialvalue=app_name)
            if name:
                self.icons_tree.insert('', tk.END, values=(
                    f'{{group}}\\{name}',
                    f'{{app}}\\{exe_name}',
                    '', '', ''
                ))
    
    def _add_desktop_icon(self):
        app_name = self.app_name_var.get()
        exe_path = filedialog.askopenfilename(
            title='选择目标程序',
            filetypes=[('可执行文件', '*.exe'), ('所有文件', '*.*')]
        )
        if exe_path:
            exe_name = os.path.basename(exe_path)
            name = simpledialog.askstring('快捷方式名称', 
                '请输入桌面快捷方式名称：', initialvalue=app_name)
            if name:
                self.icons_tree.insert('', tk.END, values=(
                    f'{{autodesktop}}\\{name}',
                    f'{{app}}\\{exe_name}',
                    '', '', ''
                ))
    
    def _add_icon(self):
        name = simpledialog.askstring('快捷方式名称', '请输入快捷方式名称（例如 {group}\\我的程序）：')
        if name:
            filename = filedialog.askopenfilename(filetypes=[('可执行文件', '*.exe')])
            if filename:
                self.icons_tree.insert('', tk.END, values=(name, filename, '', '', ''))
    
    def _remove_icon(self):
        selected = self.icons_tree.selection()
        for item in selected:
            self.icons_tree.delete(item)
    
    def _load_license_file(self):
        file_path = filedialog.askopenfilename(
            title='选择协议文件',
            filetypes=[('文本文件', '*.txt'), ('所有文件', '*.*')]
        )
        if file_path:
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                self.license_text_widget.delete(1.0, tk.END)
                self.license_text_widget.insert(tk.END, content)
            except Exception as e:
                messagebox.showerror('错误', f'加载协议失败：{str(e)}')
    
    def _save_license_file(self):
        file_path = filedialog.asksaveasfilename(
            title='保存协议为',
            defaultextension='.txt',
            filetypes=[('文本文件', '*.txt')]
        )
        if file_path:
            try:
                content = self.license_text_widget.get(1.0, tk.END)
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                messagebox.showinfo('完成', '协议保存成功。')
            except Exception as e:
                messagebox.showerror('错误', f'保存失败：{str(e)}')
    
    def _restore_license(self):
        if messagebox.askyesno('确认', '确定要恢复默认协议文本吗？'):
            self.license_text_widget.delete(1.0, tk.END)
            self.license_text_widget.insert(tk.END, self._default_license())
    
    def _open_script(self):
        file_path = filedialog.askopenfilename(filetypes=[('安装脚本', '*.iss'), ('所有文件', '*.*')])
        if file_path:
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                self.script_text.delete(1.0, tk.END)
                self.script_text.insert(tk.END, content)
                self.current_script = file_path
                self._parse_script(content)
                self.status_bar.config(text=f'已打开：{file_path}')
            except Exception as e:
                messagebox.showerror('错误', f'打开脚本失败：{str(e)}')
    
    def _parse_script(self, content):
        import tempfile
        temp_file = tempfile.mktemp(suffix='.iss')
        try:
            with open(temp_file, 'w', encoding='utf-8') as f:
                f.write(content)
            
            parser = ScriptParser()
            parser.parse_file(temp_file)
            
            config = parser.get_setup_config()
            self.app_name_var.set(config.get('AppName', 'My Program'))
            self.app_version_var.set(config.get('AppVersion', '1.0'))
            self.publisher_var.set(config.get('AppPublisher', 'My Company'))
            self.default_dir_var.set(config.get('DefaultDirName', r'C:\Program Files\My Program'))
            self.default_group_var.set(config.get('DefaultGroupName', 'My Program'))
            self.uninstall_icon_var.set(config.get('UninstallDisplayIcon', r'{app}\MyProg.exe'))
            self.compression_var.set(config.get('Compression', 'zip'))
            self.output_dir_var.set(config.get('OutputDir', '.'))
            
            if 'LicenseText' in config:
                self.license_text_widget.delete(1.0, tk.END)
                self.license_text_widget.insert(tk.END, config['LicenseText'])
            
            for item in self.files_tree.get_children():
                self.files_tree.delete(item)
            
            for file_entry in parser.get_files():
                source = file_entry.get('Source', '')
                dest_dir = file_entry.get('DestDir', '{app}')
                dest_name = file_entry.get('DestName', '')
                flags = file_entry.get('Flags', '')
                self.files_tree.insert('', tk.END, values=(source, dest_dir, dest_name, flags))
            
            for item in self.icons_tree.get_children():
                self.icons_tree.delete(item)
            
            for icon_entry in parser.get_icons():
                name = icon_entry.get('Name', '')
                filename = icon_entry.get('Filename', '')
                params = icon_entry.get('Parameters', '')
                working_dir = icon_entry.get('WorkingDir', '')
                icon_file = icon_entry.get('IconFile', '')
                self.icons_tree.insert('', tk.END, values=(name, filename, params, working_dir, icon_file))
        finally:
            if os.path.exists(temp_file):
                os.remove(temp_file)
    
    def _sync_from_script_tab(self):
        content = self.script_text.get(1.0, tk.END).strip()
        if content:
            try:
                self._parse_script(content)
            except Exception as e:
                messagebox.showerror('错误', f'解析脚本失败：{str(e)}')

    def _on_tab_changed(self, event=None):
        current_index = self.notebook.index(self.notebook.select())
        if self._last_tab_index == self.script_tab_index and current_index != self.script_tab_index:
            self._sync_from_script_tab()
        if current_index == self.script_tab_index:
            script = self._generate_script()
            self.script_text.delete(1.0, tk.END)
            self.script_text.insert(tk.END, script)
        self._last_tab_index = current_index

    def _save_script(self):
        if self.notebook.index(self.notebook.select()) == self.script_tab_index:
            self._sync_from_script_tab()
        script = self._generate_script()
        if self.current_script:
            try:
                with open(self.current_script, 'w', encoding='utf-8') as f:
                    f.write(script)
                self.status_bar.config(text=f'已保存：{self.current_script}')
            except Exception as e:
                messagebox.showerror('错误', f'保存脚本失败：{str(e)}')
        else:
            file_path = filedialog.asksaveasfilename(defaultextension='.iss', 
                                                    filetypes=[('安装脚本', '*.iss')])
            if file_path:
                try:
                    with open(file_path, 'w', encoding='utf-8') as f:
                        f.write(script)
                    self.current_script = file_path
                    self.status_bar.config(text=f'已保存：{file_path}')
                except Exception as e:
                    messagebox.showerror('错误', f'保存脚本失败：{str(e)}')
        
        self.script_text.delete(1.0, tk.END)
        self.script_text.insert(tk.END, script)
    
    def _generate_script(self):
        license_text = self.license_text_widget.get(1.0, tk.END).rstrip()
        
        script = f'''; -- Generated by Trae Installer Creator --

[Setup]
AppName={self.app_name_var.get()}
AppVersion={self.app_version_var.get()}
AppPublisher={self.publisher_var.get()}
DefaultDirName={self.default_dir_var.get()}
DefaultGroupName={self.default_group_var.get()}
UninstallDisplayIcon={self.uninstall_icon_var.get()}
Compression={self.compression_var.get()}
OutputDir={self.output_dir_var.get()}
LicenseText={license_text.replace(chr(10), '\\n').replace(chr(13), '')}

[Files]
'''
        
        for item in self.files_tree.get_children():
            values = self.files_tree.item(item, 'values')
            source, dest_dir, dest_name, flags = values
            line = f'Source: "{source}"; DestDir: "{dest_dir}"'
            if dest_name:
                line += f'; DestName: "{dest_name}"'
            if flags:
                line += f'; Flags: {flags}'
            script += line + '\n'
        
        script += '\n[Icons]\n'
        
        for item in self.icons_tree.get_children():
            values = self.icons_tree.item(item, 'values')
            name, filename, params, working_dir, icon_file = values
            line = f'Name: "{name}"; Filename: "{filename}"'
            if params:
                line += f'; Parameters: "{params}"'
            if working_dir:
                line += f'; WorkingDir: "{working_dir}"'
            if icon_file:
                line += f'; IconFile: "{icon_file}"'
            script += line + '\n'
        
        return script
    
    def _build_setup(self):
        if self.notebook.index(self.notebook.select()) == self.script_tab_index:
            self._sync_from_script_tab()
        script = self._generate_script()
        
        import tempfile
        temp_file = tempfile.mktemp(suffix='.iss')
        try:
            with open(temp_file, 'w', encoding='utf-8') as f:
                f.write(script)
        except Exception as e:
            messagebox.showerror('错误', f'写入临时文件失败：{str(e)}')
            return
        
        output_name = self.output_name_var.get()
        if not output_name:
            output_name = f'{self.app_name_var.get()} Setup {self.app_version_var.get()}.exe'
        
        output_path = os.path.join(self.output_dir_var.get(), output_name)
        
        self.status_bar.config(text='正在生成安装包...')
        self.root.update_idletasks()
        
        def build_thread():
            try:
                self.builder.load_script(temp_file)
                result = self.builder.build(output_path)
                
                self.root.after(0, lambda: self._build_complete(result))
            except Exception as e:
                self.root.after(0, lambda: self._build_error(str(e), traceback.format_exc()))
            finally:
                if os.path.exists(temp_file):
                    os.remove(temp_file)
        
        threading.Thread(target=build_thread, daemon=True).start()
    
    def _build_complete(self, result):
        self.status_bar.config(text='生成完成！')
        messagebox.showinfo('生成成功', 
            f'安装包生成成功！\n\n'
            f'输出文件夹：{result}\n\n'
            f'注意：这是一个文件夹形式的安装包。\n'
            f'要运行安装程序，请打开文件夹，双击里面的 .exe 文件。')
    
    def _build_error(self, error, trace):
        self.status_bar.config(text='生成失败')
        messagebox.showerror('错误', f'生成失败：\n{error}\n\n{trace}')


def main():
    try:
        root = tk.Tk()
        app = SetupCreatorGUI(root)
        root.mainloop()
    except Exception as e:
        try:
            messagebox.showerror('致命错误', f'程序启动失败：\n{str(e)}')
        except:
            print(f'致命错误：{str(e)}')
            traceback.print_exc()


if __name__ == '__main__':
    main()
