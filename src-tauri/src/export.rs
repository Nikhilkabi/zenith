use crate::db::DbError;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub fn zip_library(library_root: &Path, dest: &Path) -> Result<(), DbError> {
    if dest.starts_with(library_root) {
        return Err(DbError::msg(
            "Save the zip outside the Bucket library folder.",
        ));
    }

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = File::create(dest)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    add_dir(&mut zip, library_root, library_root, options)?;
    zip.finish().map_err(|e| DbError::msg(e.to_string()))?;
    Ok(())
}

fn add_dir(
    zip: &mut ZipWriter<File>,
    root: &Path,
    current: &Path,
    options: SimpleFileOptions,
) -> Result<(), DbError> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .strip_prefix(root)
            .map_err(|_| DbError::msg("Path is outside library root"))?
            .to_string_lossy()
            .replace('\\', "/");

        if path.is_dir() {
            if !name.is_empty() {
                zip.add_directory(&name, options)
                    .map_err(|e| DbError::msg(e.to_string()))?;
            }
            add_dir(zip, root, &path, options)?;
        } else {
            zip.start_file(&name, options)
                .map_err(|e| DbError::msg(e.to_string()))?;
            let mut file = File::open(&path)?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            zip.write_all(&buf)?;
        }
    }
    Ok(())
}

pub fn import_library_zip(library_root: &Path, zip_path: &Path) -> Result<u32, DbError> {
    let file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| DbError::msg(e.to_string()))?;
    let mut copied = 0u32;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| DbError::msg(e.to_string()))?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().replace('\\', "/");
        if name.contains("..") || name.starts_with('/') {
            continue;
        }
        if name == "db.sqlite" || name.ends_with("/db.sqlite") {
            continue;
        }
        let dest = library_root.join(name.replace('/', std::path::MAIN_SEPARATOR_STR));
        if dest.exists() {
            continue;
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = File::create(&dest)?;
        std::io::copy(&mut entry, &mut out)?;
        copied += 1;
    }
    Ok(copied)
}
