use anyhow::{Context, Result};
use std::{fs, path::Path};

pub fn read_file(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let p = path.as_ref();
    fs::read(p).with_context(|| format!("Failed to read file: {}", p.display()))
}
