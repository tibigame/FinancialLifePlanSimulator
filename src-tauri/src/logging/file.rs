use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
};

pub fn create_log_file(directory: &Path, timestamp: &str) -> io::Result<(File, PathBuf)> {
    let directory = directory.join("log");
    fs::create_dir_all(&directory)?;
    for suffix in 0..u32::MAX {
        let name = if suffix == 0 {
            format!("{timestamp}.log")
        } else {
            format!("{timestamp}_{suffix:03}.log")
        };
        let path = directory.join(name);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((file, path)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::other("Log filename suffixes exhausted"))
}
