use crate::model::{ArchiveFileMeta, FileEntry};
use anyhow::{bail, Context, Result};
use crc32fast::Hasher;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

const IO_BUF_SIZE: usize = 64 * 1024;

/// Stream all configured files and directories into a single solid zlib stream.
pub fn pack_solid_archive<W: Write>(
    files: &[FileEntry],
    base_dir: &Path,
    archive_out: W,
) -> Result<(Vec<ArchiveFileMeta>, u64)> {
    let mut encoder = ZlibEncoder::new(BufWriter::new(archive_out), Compression::best());
    let mut metas = Vec::new();
    let mut total_bytes: u64 = 0;

    for entry in files {
        let src_path = resolve_source_path(&entry.source, base_dir);
        if !src_path.exists() {
            continue;
        }
        let rel_dest = compute_rel_dest(&entry.dest_dir);

        if src_path.is_file() {
            let file_name = if !entry.dest_name.is_empty() {
                entry.dest_name.clone()
            } else {
                src_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file.bin")
                    .to_string()
            };
            let rel_path = join_archive_rel(&rel_dest, &file_name);
            let meta = stream_file_into_encoder(&src_path, &rel_path, &mut encoder)?;
            total_bytes = total_bytes.saturating_add(meta.size);
            metas.push(meta);
        } else if src_path.is_dir() {
            let sub_base = if !entry.dest_name.is_empty() {
                join_archive_rel(&rel_dest, &entry.dest_name)
            } else {
                rel_dest.clone()
            };

            for walk_res in WalkDir::new(&src_path) {
                let walk_entry = walk_res?;
                if !walk_entry.file_type().is_file() {
                    continue;
                }
                let item_path = walk_entry.path();
                let rel_in_dir = item_path
                    .strip_prefix(&src_path)
                    .with_context(|| format!("Failed to strip prefix for {:?}", item_path))?;
                let rel_str = path_to_slash_string(rel_in_dir);
                let rel_path = join_archive_rel(&sub_base, &rel_str);
                let meta = stream_file_into_encoder(item_path, &rel_path, &mut encoder)?;
                total_bytes = total_bytes.saturating_add(meta.size);
                metas.push(meta);
            }
        }
    }

    let mut inner_writer = encoder.finish()?;
    inner_writer.flush()?;
    Ok((metas, total_bytes))
}

/// Extract a solid zlib archive stream into `install_dir`, verifying exact file sizes and CRC32.
pub fn extract_solid_archive<R, F>(
    archive_reader: R,
    files_meta: &[ArchiveFileMeta],
    install_dir: &Path,
    mut on_progress: F,
) -> Result<Vec<String>>
where
    R: Read,
    F: FnMut(usize, usize, &str),
{
    fs::create_dir_all(install_dir)?;
    let mut decoder = ZlibDecoder::new(BufReader::new(archive_reader));
    let mut installed_files = Vec::with_capacity(files_meta.len());
    let total = files_meta.len();
    let mut buf = vec![0u8; IO_BUF_SIZE];

    for (idx, meta) in files_meta.iter().enumerate() {
        let rel_path = sanitize_relative_path(&meta.rel_path)?;
        let out_path = install_dir.join(&rel_path);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut out_file = BufWriter::new(
            File::create(&out_path)
                .with_context(|| format!("Failed to create output file {:?}", out_path))?,
        );
        let mut hasher = Hasher::new();
        let mut remaining = meta.size;

        while remaining > 0 {
            let to_read = usize::try_from(remaining.min(buf.len() as u64)).unwrap_or(buf.len());
            let n = decoder.read(&mut buf[..to_read])?;
            if n == 0 {
                bail!(
                    "Unexpected EOF while extracting '{}' (remaining {} bytes)",
                    meta.rel_path,
                    remaining
                );
            }
            out_file.write_all(&buf[..n])?;
            hasher.update(&buf[..n]);
            remaining -= n as u64;
        }
        out_file.flush()?;

        let actual_crc = hasher.finalize();
        if actual_crc != meta.crc32 {
            bail!(
                "CRC32 mismatch for '{}': expected {:#010x}, got {:#010x}",
                meta.rel_path,
                meta.crc32,
                actual_crc
            );
        }

        installed_files.push(meta.rel_path.replace('/', "\\"));
        on_progress(idx + 1, total, &meta.rel_path);
    }

    // Drain any trailing zlib Adler32 trailer bytes
    let _ = io::copy(&mut decoder, &mut io::sink());
    Ok(installed_files)
}

fn stream_file_into_encoder<W: Write>(
    src_path: &Path,
    rel_path: &str,
    encoder: &mut ZlibEncoder<W>,
) -> Result<ArchiveFileMeta> {
    let mut file = BufReader::new(File::open(src_path)?);
    let mut hasher = Hasher::new();
    let mut buf = [0u8; IO_BUF_SIZE];
    let mut size: u64 = 0;

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        encoder.write_all(&buf[..n])?;
        hasher.update(&buf[..n]);
        size += n as u64;
    }

    Ok(ArchiveFileMeta {
        rel_path: rel_path.to_string(),
        size,
        crc32: hasher.finalize(),
    })
}

fn resolve_source_path(source: &str, base_dir: &Path) -> PathBuf {
    let p = Path::new(source);
    if p.is_absolute() || p.exists() {
        p.to_path_buf()
    } else {
        base_dir.join(p)
    }
}

fn compute_rel_dest(dest_dir: &str) -> String {
    let trimmed = dest_dir.trim();
    let without_app = if trimmed.len() >= 5 && trimmed[..5].eq_ignore_ascii_case("{app}") {
        &trimmed[5..]
    } else {
        trimmed
    };
    without_app
        .trim_matches(['\\', '/'])
        .replace('\\', "/")
        .trim_matches('/')
        .to_string()
}

fn join_archive_rel(prefix: &str, suffix: &str) -> String {
    let clean_suffix = suffix.replace('\\', "/").trim_start_matches('/').to_string();
    if prefix.is_empty() {
        clean_suffix
    } else if clean_suffix.is_empty() {
        prefix.to_string()
    } else {
        format!("{}/{}", prefix.trim_end_matches('/'), clean_suffix)
    }
}

fn path_to_slash_string(path: &Path) -> String {
    path.components()
        .filter_map(|c| match c {
            Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn sanitize_relative_path(rel: &str) -> Result<PathBuf> {
    let mut out = PathBuf::new();
    for seg in rel.split(['/', '\\']) {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." || seg.contains(':') {
            bail!("Unsafe archive entry path: {}", rel);
        }
        out.push(seg);
    }
    if out.as_os_str().is_empty() {
        bail!("Empty archive entry path: {}", rel);
    }
    Ok(out)
}
