use std::{fmt::Display, path::PathBuf};

#[derive(Debug, Clone)]
pub struct BlockDevice {
    /// Path to open the device
    pub path: String,

    /// Is the device removable (SD Card, external SSD/HDD, etc..)
    pub removable: bool,

    /// Is the block device a partition or a disk
    pub is_partition: bool,

    /// Static human representation of the device
    pub repr: String,

    /// Partitions of the disk (if it is a disk)
    pub partitions: Option<Vec<Self>>,
}

// just print the representation
impl Display for BlockDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.repr)
    }
}

impl BlockDevice {
    pub fn open(&self, readonly: bool) -> std::io::Result<std::fs::File> {
        std::fs::OpenOptions::new()
            .read(true)
            .write(!readonly)
            .open(&self.path)
    }
}

pub fn find_mp3_files(vec: &mut Vec<PathBuf>, path: PathBuf) -> std::io::Result<()> {
    if path.is_dir() {
        let paths = std::fs::read_dir(&path)?;
        for path_result in paths {
            let full_path = path_result?.path();
            find_mp3_files(vec, full_path)?;
        }
    } else {
        // only collect MP3 files
        if let Some(ext) = path.extension() {
            if ext == "mp3" {
                vec.push(path);
            }
        }
    }

    Ok(())
}
