import zlib
import os
import struct
from typing import List, Dict, Optional
import zipfile
import tarfile

try:
    import bz2
    HAS_BZ2 = True
except ImportError:
    HAS_BZ2 = False

try:
    import lzma
    HAS_LZMA = True
except ImportError:
    HAS_LZMA = False


class Compressor:
    def __init__(self, method: str = 'zip', level: int = 9):
        self.method = method.lower()
        self.level = level
        self.supported_methods = ['zlib', 'zip', 'tar']
        if HAS_BZ2:
            self.supported_methods.append('bzip2')
        if HAS_LZMA:
            self.supported_methods.extend(['lzma', 'lzma2'])
        
        if self.method not in self.supported_methods:
            self.method = 'zlib'
    
    def compress_file(self, filepath: str) -> bytes:
        with open(filepath, 'rb') as f:
            data = f.read()
        
        if self.method == 'zlib':
            return zlib.compress(data, self.level)
        elif self.method == 'bzip2' and HAS_BZ2:
            return bz2.compress(data, self.level)
        elif (self.method == 'lzma' or self.method == 'lzma2') and HAS_LZMA:
            return lzma.compress(data, preset=self.level)
        else:
            return zlib.compress(data, self.level)
    
    def compress_directory(self, dirpath: str, output_path: str) -> str:
        if self.method == 'zip':
            return self._compress_to_zip(dirpath, output_path)
        elif self.method == 'tar':
            return self._compress_to_tar(dirpath, output_path)
        else:
            return self._compress_to_custom(dirpath, output_path)
    
    def _compress_to_zip(self, dirpath: str, output_path: str) -> str:
        with zipfile.ZipFile(output_path, 'w', zipfile.ZIP_DEFLATED, compresslevel=self.level) as zf:
            for root, dirs, files in os.walk(dirpath):
                for file in files:
                    filepath = os.path.join(root, file)
                    arcname = os.path.relpath(filepath, dirpath)
                    zf.write(filepath, arcname)
        return output_path
    
    def _compress_to_tar(self, dirpath: str, output_path: str) -> str:
        mode = 'w:bz2' if (HAS_BZ2 and self.level > 0) else 'w'
        with tarfile.open(output_path, mode) as tf:
            tf.add(dirpath, arcname=os.path.basename(dirpath))
        return output_path
    
    def _compress_to_custom(self, dirpath: str, output_path: str) -> str:
        file_index = []
        compressed_data = b''
        
        for root, dirs, files in os.walk(dirpath):
            for file in files:
                filepath = os.path.join(root, file)
                rel_path = os.path.relpath(filepath, dirpath)
                
                with open(filepath, 'rb') as f:
                    raw_data = f.read()
                
                compressed = self.compress_file(filepath)
                offset = len(compressed_data)
                compressed_data += compressed
                
                file_index.append({
                    'path': rel_path,
                    'offset': offset,
                    'compressed_size': len(compressed),
                    'original_size': len(raw_data)
                })
        
        index_data = self._serialize_index(file_index)
        header = struct.pack('<II', len(index_data), len(compressed_data))
        
        with open(output_path, 'wb') as f:
            f.write(header)
            f.write(index_data)
            f.write(compressed_data)
        
        return output_path
    
    def _serialize_index(self, file_index: List[Dict]) -> bytes:
        index_data = b''
        for entry in file_index:
            path_bytes = entry['path'].encode('utf-8')
            index_data += struct.pack('<I', len(path_bytes))
            index_data += path_bytes
            index_data += struct.pack('<QQQ', 
                                     entry['offset'], 
                                     entry['compressed_size'], 
                                     entry['original_size'])
        return index_data
    
    def decompress_data(self, data: bytes) -> bytes:
        if self.method == 'zlib':
            return zlib.decompress(data)
        elif self.method == 'bzip2' and HAS_BZ2:
            return bz2.decompress(data)
        elif (self.method == 'lzma' or self.method == 'lzma2') and HAS_LZMA:
            return lzma.decompress(data)
        else:
            return zlib.decompress(data)
    
    def extract_from_custom(self, archive_path: str, extract_dir: str) -> None:
        with open(archive_path, 'rb') as f:
            header = f.read(8)
            index_size, data_size = struct.unpack('<II', header)
            
            index_data = f.read(index_size)
            compressed_data = f.read(data_size)
        
        file_index = self._deserialize_index(index_data)
        
        os.makedirs(extract_dir, exist_ok=True)
        
        for entry in file_index:
            extract_path = os.path.join(extract_dir, entry['path'])
            os.makedirs(os.path.dirname(extract_path), exist_ok=True)
            
            offset = entry['offset']
            size = entry['compressed_size']
            compressed = compressed_data[offset:offset+size]
            decompressed = self.decompress_data(compressed)
            
            with open(extract_path, 'wb') as f:
                f.write(decompressed)
    
    def _deserialize_index(self, index_data: bytes) -> List[Dict]:
        file_index = []
        pos = 0
        
        while pos < len(index_data):
            path_len = struct.unpack('<I', index_data[pos:pos+4])[0]
            pos += 4
            
            path = index_data[pos:pos+path_len].decode('utf-8')
            pos += path_len
            
            offset, compressed_size, original_size = struct.unpack('<QQQ', index_data[pos:pos+24])
            pos += 24
            
            file_index.append({
                'path': path,
                'offset': offset,
                'compressed_size': compressed_size,
                'original_size': original_size
            })
        
        return file_index