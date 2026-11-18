use anyhow::{bail, Result};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub const MAGIC_INSTALLER: [u8; 8] = *b"TRAESET3";
pub const MAGIC_UNINSTALLER: [u8; 8] = *b"TRAEUNI3";
pub const FOOTER_SIZE: usize = 32;

/// Fixed 32-byte footer located at EOF for O(1) payload discovery without scanning the executable.
/// Layout (little-endian):
/// - [0..8]   config_offset: u64
/// - [8..16]  config_size:   u64
/// - [16..24] archive_size:  u64
/// - [24..32] magic:         [u8; 8] ("TRAESET3" or "TRAEUNI3")
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackageFooter {
    pub config_offset: u64,
    pub config_size: u64,
    pub archive_size: u64,
    pub magic: [u8; 8],
}

impl PackageFooter {
    pub fn new(config_offset: u64, config_size: u64, archive_size: u64, magic: [u8; 8]) -> Self {
        Self {
            config_offset,
            config_size,
            archive_size,
            magic,
        }
    }

    pub fn archive_offset(&self) -> u64 {
        self.config_offset.saturating_add(self.config_size)
    }

    pub fn is_installer(&self) -> bool {
        self.magic == MAGIC_INSTALLER
    }

    pub fn is_uninstaller(&self) -> bool {
        self.magic == MAGIC_UNINSTALLER
    }

    pub fn to_bytes(&self) -> [u8; FOOTER_SIZE] {
        let mut buf = [0u8; FOOTER_SIZE];
        buf[0..8].copy_from_slice(&self.config_offset.to_le_bytes());
        buf[8..16].copy_from_slice(&self.config_size.to_le_bytes());
        buf[16..24].copy_from_slice(&self.archive_size.to_le_bytes());
        buf[24..32].copy_from_slice(&self.magic);
        buf
    }

    pub fn from_bytes(buf: &[u8; FOOTER_SIZE]) -> Option<Self> {
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&buf[24..32]);
        if magic != MAGIC_INSTALLER && magic != MAGIC_UNINSTALLER {
            return None;
        }
        let config_offset = u64::from_le_bytes(buf[0..8].try_into().ok()?);
        let config_size = u64::from_le_bytes(buf[8..16].try_into().ok()?);
        let archive_size = u64::from_le_bytes(buf[16..24].try_into().ok()?);
        Some(Self {
            config_offset,
            config_size,
            archive_size,
            magic,
        })
    }

    pub fn read_from_file(file: &mut File) -> Result<Option<Self>> {
        let file_len = file.metadata()?.len();
        if file_len < FOOTER_SIZE as u64 {
            return Ok(None);
        }
        file.seek(SeekFrom::End(-(FOOTER_SIZE as i64)))?;
        let mut buf = [0u8; FOOTER_SIZE];
        file.read_exact(&mut buf)?;

        let Some(footer) = Self::from_bytes(&buf) else {
            return Ok(None);
        };

        let expected_end = footer
            .config_offset
            .checked_add(footer.config_size)
            .and_then(|v| v.checked_add(footer.archive_size))
            .and_then(|v| v.checked_add(FOOTER_SIZE as u64));

        if expected_end != Some(file_len) {
            bail!(
                "Package footer bounds mismatch: expected total length {:?}, actual {}",
                expected_end,
                file_len
            );
        }

        Ok(Some(footer))
    }
}

/// Reads only the PE stub portion (0..config_offset) if the file already contains a footer,
/// or the entire file if it is a clean stub executable.
pub fn extract_pure_stub_bytes(exe_path: &Path) -> Result<Vec<u8>> {
    let mut file = File::open(exe_path)?;
    let total_len = file.metadata()?.len();
    let stub_len = match PackageFooter::read_from_file(&mut file)? {
        Some(footer) => footer.config_offset,
        None => total_len,
    };

    file.seek(SeekFrom::Start(0))?;
    let mut stub_bytes = vec![0u8; usize::try_from(stub_len)?];
    file.read_exact(&mut stub_bytes)?;
    Ok(stub_bytes)
}
