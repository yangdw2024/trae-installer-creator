import re
import os
from typing import Dict, List, Any, Optional


class ScriptSection:
    def __init__(self, name: str):
        self.name = name
        self.entries: List[Dict[str, str]] = []
    
    def add_entry(self, entry: Dict[str, str]):
        self.entries.append(entry)


class ScriptParser:
    def __init__(self):
        self.sections: Dict[str, ScriptSection] = {}
        self.current_section: Optional[ScriptSection] = None
    
    def parse_file(self, filepath: str) -> Dict[str, ScriptSection]:
        self.sections = {}
        self.current_section = None
        
        with open(filepath, 'r', encoding='utf-8') as f:
            lines = f.readlines()
        
        for line in lines:
            line = line.strip()
            if not line or line.startswith(';') or line.startswith('//'):
                continue
            
            section_match = re.match(r'\[(\w+)\]', line)
            if section_match:
                section_name = section_match.group(1)
                self.current_section = ScriptSection(section_name)
                self.sections[section_name] = self.current_section
                continue
            
            if self.current_section:
                if self.current_section.name.lower() in ('setup', 'license'):
                    entry = self._parse_key_value_entry(line)
                else:
                    entry = self._parse_entry(line)
                if entry:
                    self.current_section.add_entry(entry)
        
        return self.sections
    
    def _parse_key_value_entry(self, line: str) -> Optional[Dict[str, str]]:
        match = re.match(r'^(\w+)\s*[=:]\s*(.*)$', line)
        if not match:
            return None
        key = match.group(1)
        value = match.group(2).strip()
        if len(value) >= 2 and (
            (value.startswith('"') and value.endswith('"')) or
            (value.startswith("'") and value.endswith("'"))
        ):
            value = value[1:-1]
        if key.lower() == 'licensetext':
            value = value.replace('\\n', '\n')
        return {key: value}
    
    def _parse_entry(self, line: str) -> Optional[Dict[str, str]]:
        entry = {}
        parts = re.split(r'(?<!\\);', line)
        
        for part in parts:
            part = part.strip()
            if not part:
                continue
            
            key_value_match = re.match(r'^(\w+)\s*[=:]\s*(.+)$', part, re.IGNORECASE)
            if key_value_match:
                key = key_value_match.group(1)
                value = key_value_match.group(2).strip()
                if (value.startswith('"') and value.endswith('"')) or \
                   (value.startswith("'") and value.endswith("'")):
                    value = value[1:-1]
                entry[key] = value
            else:
                entry[part] = 'yes'
        
        return entry if entry else None
    
    def get_section(self, name: str) -> Optional[ScriptSection]:
        return self.sections.get(name)
    
    def get_setup_config(self) -> Dict[str, str]:
        config = {}
        for target_name in ('setup', 'license'):
            for sec_key, section in self.sections.items():
                if sec_key.lower() == target_name:
                    for entry in section.entries:
                        for key, value in entry.items():
                            config[key] = value
        return config
    
    def get_files(self) -> List[Dict[str, str]]:
        files = []
        files_section = self.sections.get('Files')
        if files_section:
            for entry in files_section.entries:
                files.append(entry)
        return files
    
    def get_icons(self) -> List[Dict[str, str]]:
        icons = []
        icons_section = self.sections.get('Icons')
        if icons_section:
            for entry in icons_section.entries:
                icons.append(entry)
        return icons
    
    def get_components(self) -> List[Dict[str, str]]:
        components = []
        components_section = self.sections.get('Components')
        if components_section:
            for entry in components_section.entries:
                components.append(entry)
        return components
    
    def get_tasks(self) -> List[Dict[str, str]]:
        tasks = []
        tasks_section = self.sections.get('Tasks')
        if tasks_section:
            for entry in tasks_section.entries:
                tasks.append(entry)
        return tasks
    
    def get_code(self) -> str:
        code = ''
        code_section = self.sections.get('Code')
        if code_section:
            code = '\n'.join(str(e) for e in code_section.entries)
        return code