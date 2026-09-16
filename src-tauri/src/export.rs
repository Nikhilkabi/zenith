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
